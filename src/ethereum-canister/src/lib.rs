use candid::Nat;
use ic_cdk::{init, query, update};
use interface::{Address, Erc20OwnerOfRequest, Erc721OwnerOfRequest, SetupRequest, U256};

mod erc20;
mod erc721;
mod helios;
mod random;
mod utils;

#[init]
async fn init() {
    ic_cdk::setup();
}

/// Setup the helios client with given node urls
///
///
/// dfx canister call ethereum_canister setup \
///     'record { execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.org" }'
#[update]
async fn setup(request: SetupRequest) {
    let _ = ic_logger::init_with_level(log::Level::Trace);

    helios::start_client(&request.consensus_rpc_url, &request.execution_rpc_url)
        .await
        .expect("starting client failed");
}

#[query]
async fn get_block_number() -> Nat {
    let helios = helios::client();

    let head_block_num = helios
        .get_block_number()
        .await
        .expect("get_block_number failed");

    head_block_num.into()
}

#[update]
async fn erc20_balance_of(request: Erc20OwnerOfRequest) -> U256 {
    erc20::balance_of(request.contract.into(), request.account.into())
        .await
        .expect("erc20::balance_of failed")
        .into()
}

#[update]
async fn erc721_owner_of(request: Erc721OwnerOfRequest) -> Address {
    erc721::owner_of(request.contract.into(), request.token_id.into())
        .await
        .expect("erc721::owner_of failed")
        .into()
}
