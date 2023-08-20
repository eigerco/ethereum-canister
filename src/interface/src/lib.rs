use candid::CandidType;
use serde::Deserialize;

mod address;
mod network;
mod u256;

pub use address::Address;
pub use network::{BadNetwork, Network};
pub use u256::{U256ConvertError, U256};

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct SetupRequest {
    pub network: Network,
    pub consensus_rpc_url: String,
    pub execution_rpc_url: String,
    pub checkpoint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct Erc20BalanceOfRequest {
    pub contract: Address,
    pub account: Address,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct Erc721OwnerOfRequest {
    pub contract: Address,
    pub token_id: U256,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct EstimateGasRequest {
    pub from: Option<Address>,
    pub to: Address,
    pub gas_limit: Option<U256>,
    pub gas_price: Option<U256>,
    pub value: Option<U256>,
    pub data: Option<Vec<u8>>,
}
