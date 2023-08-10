use std::fs::File;

use candid::{utils::ArgumentEncoder, IDLArgs};
use eyre::{ensure, Result, WrapErr};
use serde_json::{json, Value};
use temp_dir::TempDir;

static DFX_JSON: &str = include_str!("../../../../dfx.json");

const DEFAULT_CONSENSUS_RPC: &str = "https://www.lightclientdata.org";
const DEFAULT_EXECUTION_RPC: &str = "https://ethereum.publicnode.com";

// NOTE: we can't just decode from inside the `call` because decode_args
// returns ArgumentDecoder<'a> where 'a lifetime would be tied to the output we're
// parsing, which would be dropped on return.
// The reason for this is that ArgumentDecoder allows decoding by ref eg. to (&str,)
macro_rules! call_decode {
    ($canister:expr, $method:expr, $args:expr) => {{
        $canister.call($method, $args).and_then(|output| {
            ::candid::utils::decode_args(&output).map_err(|err| ::eyre::eyre!(err))
        })
    }};
}

pub(crate) use call_decode;

#[derive(Debug)]
pub struct TestCanister {
    name: String,
    temp_dir: TempDir,
}

impl TestCanister {
    pub fn deploy() -> Self {
        let temp_dir = TempDir::new().unwrap();

        // setup the tempdir
        make_symlink(&temp_dir, "src");
        make_symlink(&temp_dir, "target");
        make_symlink(&temp_dir, "Cargo.toml");
        make_symlink(&temp_dir, "Cargo.lock");

        let name = format!("ethereum_canister_{}", rand::random::<u64>());

        // rename the canister in dfx.json
        let mut dfx_config: Value = serde_json::from_str(DFX_JSON).unwrap();
        let ethereum_canister = dfx_config["canisters"]["ethereum_canister"].take();
        dfx_config["canisters"] = json!({ name.as_str(): ethereum_canister });

        // create dfx.json
        let dfx_json = File::create(temp_dir.child("dfx.json")).unwrap();
        serde_json::to_writer_pretty(dfx_json, &dfx_config).unwrap();

        // deploy
        let canister = TestCanister { temp_dir, name };
        canister.run_dfx(&["deploy"]).unwrap();

        canister
    }

    pub fn deploy_and_setup() -> Self {
        let canister = Self::deploy();
        let request = interface::SetupRequest {
            consensus_rpc_url: DEFAULT_CONSENSUS_RPC.to_owned(),
            execution_rpc_url: DEFAULT_EXECUTION_RPC.to_owned(),
        };
        canister.call("setup", (request,)).unwrap();
        canister
    }

    pub fn call<Args: ArgumentEncoder>(&self, method: &str, args: Args) -> Result<Vec<u8>> {
        let args = candid::utils::encode_args(args).wrap_err("encoding args")?;
        let args = IDLArgs::from_bytes(&args).wrap_err("decoding args")?;

        let stdout = self
            .run_dfx(&["canister", "call", &self.name, method, &args.to_string()])
            .wrap_err(format!("calling '{method} {args}'"))?;

        let stdout = std::str::from_utf8(&stdout).wrap_err("decoding output")?;
        let output: IDLArgs = stdout.parse().wrap_err("parsing output")?;

        output.to_bytes().wrap_err("encoding to candid")
    }

    fn remove(&self) {
        self.run_dfx(&["canister", "stop", &self.name])
            .expect("Stopping failed");
        self.run_dfx(&["canister", "delete", &self.name])
            .expect("Deleting failed");
    }

    fn run_dfx(&self, args: &[&str]) -> Result<Vec<u8>> {
        let output = std::process::Command::new("dfx")
            .args(args)
            .current_dir(self.temp_dir.path())
            .output()
            .wrap_err(format!("executing dfx {args:?}"))?;
        ensure!(
            output.status.success(),
            "dfx {args:?} failed: {}",
            std::str::from_utf8(&output.stderr)?
        );
        Ok(output.stdout)
    }
}

impl Drop for TestCanister {
    fn drop(&mut self) {
        self.remove()
    }
}

fn make_symlink(temp_dir: &TempDir, name: &str) {
    #[cfg(not(target_family = "unix"))]
    {
        _ = temp_dir;
        _ = name;
        panic!("unsupported test target")
    }
    #[cfg(target_family = "unix")]
    {
        let path = format!("{}/../../{name}", env!("CARGO_MANIFEST_DIR"));
        std::os::unix::fs::symlink(path, temp_dir.child(name)).unwrap();
    }
}
