use candid::CandidType;
use serde::Deserialize;

mod address;
mod u256;

pub use address::Address;
pub use u256::{U256ConvertError, U256};

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct SetupRequest {
    pub consensus_rpc_url: String,
    pub execution_rpc_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct Erc20OwnerOfRequest {
    pub contract: Address,
    pub account: Address,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct Erc721OwnerOfRequest {
    pub contract: Address,
    pub token_id: U256,
}
