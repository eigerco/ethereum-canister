use std::cell::RefCell;
use std::rc::Rc;

use candid::Nat;
use ethers_core::types::Address;
use helios_client::database::ConfigDB;
use helios_client::{Client, ClientBuilder};
use helios_config::Network;
use ic_cdk::{init, query, update};

mod erc20;
mod random;
mod utils;

use crate::utils::ToNat;

thread_local! {
    static HELIOS: RefCell<Option<Rc<Client<ConfigDB>>>> = RefCell::new(None);
}

pub(crate) fn global_client() -> Rc<Client<ConfigDB>> {
    HELIOS
        .with(|helios| helios.borrow().clone())
        .expect("Client not initialized")
}

#[init]
async fn init() {
    ic_cdk::setup();
}

// Note: example call for test purposes (remove me)
// the first url is pointed by 301 MOVED from https://www.lightclientdata.org
// dfx canister call ethereum_canister_backend setup \
// '("https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/", "https://ethereum.publicnode.com", "0x2196fc70451d54e95061bfc2d756f3a8cf6e243f78dd475a8793da6afd17b423")'
#[update]
async fn setup(consensus_rpc_url: String, execution_rpc_url: String, checkpoint: String) {
    let _ = ic_logger::init_with_level(log::Level::Trace);

    let mut client: Client<ConfigDB> = ClientBuilder::new()
        .network(Network::MAINNET)
        .consensus_rpc(&consensus_rpc_url)
        .execution_rpc(&execution_rpc_url)
        .checkpoint(&checkpoint)
        .load_external_fallback()
        .build()
        .expect("Client setup failed");

    client.start().await.expect("Failed to start the client");

    HELIOS.with(|helios| *helios.borrow_mut() = Some(Rc::new(client)));
}

#[query]
async fn get_block_number() -> Nat {
    let helios = global_client();

    let head_block_num = helios
        .get_block_number()
        .await
        .expect("get_block_number failed");

    head_block_num.into()
}

#[update]
async fn erc20_balance_of(erc20: String, wallet: String) -> Nat {
    let erc20 = erc20
        .parse::<Address>()
        .expect("failed to parse erc20 address");
    let wallet = wallet
        .parse::<Address>()
        .expect("failed to parse wallet address");

    let amount = erc20::balance_of(erc20, wallet)
        .await
        .expect("erc20::balance_of failed");

    amount.to_nat()
}
