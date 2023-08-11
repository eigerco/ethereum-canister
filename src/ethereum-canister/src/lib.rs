use std::cell::RefCell;

use candid::Nat;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer;
use interface::{Address, Erc20OwnerOfRequest, Erc721OwnerOfRequest, Network, SetupRequest, U256};
use log::{debug, error};

use crate::stable_memory::{
    init_stable_cell_default, load_static_string, save_static_string, StableCell,
    LAST_CHECKPOINT_ID, LAST_CONSENSUS_RPC_URL_ID, LAST_EXECUTION_RPC_URL_ID, LAST_NETWORK_ID,
};

mod erc20;
mod erc721;
mod helios;
mod random;
mod stable_memory;
mod utils;

thread_local! {
    static LAST_NETWORK: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_NETWORK_ID));
    static LAST_CONSENSUS_RPC_URL: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_CONSENSUS_RPC_URL_ID));
    static LAST_EXECUTION_RPC_URL: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_EXECUTION_RPC_URL_ID));
    static LAST_CHECKPOINT: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_CHECKPOINT_ID));
}

#[init]
async fn init() {
    ic_cdk::setup();
}

/// Setup the helios client with given node urls
///
/// Mainnet:
///   dfx canister call ethereum_canister setup \
///     'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.org" }'
///
/// Goerli:
///   dfx canister call ethereum_canister setup \
///     'record { network = variant { Goerli }; execution_rpc_url = "https://ethereum-goerli.publicnode.com"; consensus_rpc_url = "TODO" }'
#[update]
async fn setup(request: SetupRequest) {
    let _ = ic_logger::init_with_level(log::Level::Trace);

    helios::start_client(
        request.network,
        &request.consensus_rpc_url,
        &request.execution_rpc_url,
        None,
    )
    .await
    .expect("starting client failed");

    save_static_string(&LAST_NETWORK, request.network.to_string());
    save_static_string(&LAST_CONSENSUS_RPC_URL, request.consensus_rpc_url);
    save_static_string(&LAST_EXECUTION_RPC_URL, request.execution_rpc_url);
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

#[pre_upgrade]
async fn pre_upgrade() {
    debug!("Stopping client");

    let checkpoint = helios::get_last_checkpoint().await;
    save_static_string(&LAST_CHECKPOINT, checkpoint);

    helios::shutdown().await;

    debug!("Client stopped");
}

#[post_upgrade]
async fn post_upgrade() {
    let _ = ic_logger::init_with_level(log::Level::Trace);

    // Workaround because cross-canister calls are not allowed in post_upgrade.
    // Client will be started from a timer in a second.
    set_timer(std::time::Duration::from_secs(1), || {
        ic_cdk::spawn(async move {
            let Some(network) = load_static_string(&LAST_NETWORK) else {
                return
            };

            let Ok(network) = network.parse::<Network>() else {
                error!("Failed to parse network: {network}. Use `setup` to initalize canister.");
                return
            };

            let Some(consensus_rpc_url) = load_static_string(&LAST_CONSENSUS_RPC_URL) else {
                return
            };

            let Some(execution_rpc_url) = load_static_string(&LAST_EXECUTION_RPC_URL) else {
                return
            };

            let checkpoint = load_static_string(&LAST_CHECKPOINT);

            debug!(
                "Resuming client with: network = {}, execution_rpc_url = {}, consensus_rpc_url = {}, checkpoint: {}",
                network,
                &execution_rpc_url,
                &consensus_rpc_url,
                &checkpoint.as_deref().unwrap_or("None"),
            );

            helios::start_client(
                network,
                &consensus_rpc_url,
                &execution_rpc_url,
                checkpoint.as_deref(),
            )
            .await
            .expect("starting client failed");
        });
    });
}
