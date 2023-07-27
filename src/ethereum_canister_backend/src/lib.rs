use std::cell::RefCell;
use std::rc::Rc;

use candid::{CandidType, Nat};
use helios_client::database::ConfigDB;
use helios_client::{Client, ClientBuilder};
use helios_config::Network;
use ic_cdk::{init, update};

mod random;

thread_local! {
    static HELIOS: RefCell<Option<Rc<Client<ConfigDB>>>> = RefCell::new(None);
}

#[derive(Debug, CandidType, thiserror::Error)]
enum EthClientError {
    #[error("Couldn't set up the client")]
    SetupFailed,

    #[error("Client not initialized")]
    NotInitialized,

    #[error("Rpc method failed: {0}")]
    RpcFailed(String),
}

type Result<T> = std::result::Result<T, EthClientError>;

#[init]
async fn init() {
    ic_cdk::setup();
}

// Note: example call for test purposes (remove me)
// the first url is pointed by 301 MOVED from https://www.lightclientdata.org
// dfx canister call ethereum_canister_backend setup \
// '("https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/", "https://ethereum.publicnode.com", "0x2196fc70451d54e95061bfc2d756f3a8cf6e243f78dd475a8793da6afd17b423")'
#[update]
async fn setup(
    consensus_rpc_url: String,
    execution_rpc_url: String,
    checkpoint: String,
) -> Result<()> {
    let mut client: Client<ConfigDB> = ClientBuilder::new()
        .network(Network::MAINNET)
        .consensus_rpc(&consensus_rpc_url)
        .execution_rpc(&execution_rpc_url)
        .checkpoint(&checkpoint)
        .load_external_fallback()
        .build()
        .map_err(|err| {
            ic_cdk::println!("Client setup failed: {err}");
            EthClientError::SetupFailed
        })?;

    client.start().await.map_err(|err| {
        ic_cdk::println!("Failed to start the client: {err}");
        EthClientError::SetupFailed
    })?;

    HELIOS.with(|helios| *helios.borrow_mut() = Some(Rc::new(client)));

    Ok(())
}

#[update]
async fn get_block_number() -> Result<Nat> {
    let head_block_num = HELIOS
        .with(|client| client.borrow().clone())
        .ok_or(EthClientError::NotInitialized)?
        .get_block_number()
        .await
        .map_err(|err| {
            ic_cdk::println!("Get block number failed: {err}");
            EthClientError::RpcFailed("get_block_number".into())
        })?;

    Ok(head_block_num.into())
}

// #[update]
// async fn greet() -> String {
//     let _guard = random::enter().await;

//     let n: u32 = rand::random();
//     format!("Hello, here is a random number: {}", n)
// }
