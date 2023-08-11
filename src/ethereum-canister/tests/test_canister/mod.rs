use candid::{utils::ArgumentEncoder, IDLArgs};
use eyre::{ensure, Result, WrapErr};
use temp_dir::TempDir;

const DEFAULT_CONSENSUS_RPC: &str = "https://www.lightclientdata.org";
const DEFAULT_EXECUTION_RPC: &str = "https://ethereum.publicnode.com";

#[derive(Debug)]
pub struct TestCanister {
    name: String,
    temp_dir: TempDir,
}

impl TestCanister {
    pub fn deploy(name: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();

        // setup the tempdir
        make_symlink(&temp_dir, "src");
        make_symlink(&temp_dir, "target");
        make_symlink(&temp_dir, "Cargo.toml");
        make_symlink(&temp_dir, "Cargo.lock");
        make_symlink(&temp_dir, "dfx.json");

        // deploy
        let canister = TestCanister {
            temp_dir,
            name: name.to_owned(),
        };
        canister.run_dfx(&["deploy", name]).unwrap();

        canister
    }

    pub fn call(&self, method: &str, args: impl ArgumentEncoder) -> Result<Vec<u8>> {
        // convert arguments into format understood by `dfx`
        let args = candid::utils::encode_args(args).wrap_err("encoding args")?;
        let args = IDLArgs::from_bytes(&args).wrap_err("decoding dfx args")?;

        let stdout = self
            .run_dfx(&["canister", "call", &self.name, method, &args.to_string()])
            .wrap_err_with(|| format!("calling '{method} {args}'"))?;

        // convert results from the format understood by `dfx`
        // to the candid binary representation
        // note: decoding here to the correct type is not possible as decoding
        //       binds the result's lifetime to the lifetime of output
        //       this is because decoding supports also reference types
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
            .wrap_err_with(|| format!("executing dfx {args:?}"))?;
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

pub fn setup_ethereum_canister() -> TestCanister {
    let canister = TestCanister::deploy("ethereum_canister");
    let request = interface::SetupRequest {
        consensus_rpc_url: DEFAULT_CONSENSUS_RPC.to_owned(),
        execution_rpc_url: DEFAULT_EXECUTION_RPC.to_owned(),
    };
    let _: () = call!(canister, "setup", request).unwrap();
    canister
}

/// A helper macro that allows calling canister methods with single and multiple arguments
/// and decodes the results.
macro_rules! call {
    ($canister:expr, $method:expr, ($($arg:expr),*)) => {{
        let result = $canister.call($method, ($($arg),*));
        crate::test_canister::call!(@decode, result)

    }};
    ($canister:expr, $method:expr, $arg:expr) => {{
        let result = $canister.call($method, ($arg,));
        crate::test_canister::call!(@decode, result)
    }};
    ($canister:expr, $method:expr) => {{
        let result = $canister.call($method, ());
        crate::test_canister::call!(@decode, result)
    }};
    (@decode, $result:expr) => {
        $result.and_then(|output| {
            ::candid::utils::decode_args(&output).map_err(|err| ::eyre::eyre!(err))
        })
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

// this has to come after macro definitions
pub(crate) use call;
