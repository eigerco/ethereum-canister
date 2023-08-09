use candid::Nat;
use ic_cdk::{init, query, update};

mod erc20;
mod erc721;
mod helios;
mod random;
mod utils;

use crate::utils::ToNat;

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

    helios::start_client(&consensus_rpc_url, &execution_rpc_url, &checkpoint)
        .await
        .unwrap();
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
async fn erc20_balance_of(erc20_contract: String, account: String) -> Nat {
    let contract = erc20_contract
        .parse()
        .expect("failed to parse erc20_contract address");
    let account = account.parse().expect("failed to parse account address");

    let amount = erc20::balance_of(contract, account)
        .await
        .expect("erc20::balance_of failed");

    amount.to_nat()
}

#[update]
async fn erc721_owner_of(erc721_contract: String, token_id: String) -> String {
    let contract = erc721_contract
        .parse()
        .expect("Failed to parse erc721_contract address");
    let token_id = token_id.parse().expect("Failed to parse token_id address");

    let owner = erc721::owner_of(contract, token_id)
        .await
        .expect("erc721::owner_of failed");

    format!("{:?}", owner)
}
