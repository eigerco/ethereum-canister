use std::fmt::{self, Display};
use std::str::FromStr;

use candid::CandidType;
use serde::Deserialize;
use thiserror::Error;

mod address;
mod u256;

pub use address::Address;
pub use u256::{U256ConvertError, U256};

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct SetupRequest {
    pub network: Network,
    pub consensus_rpc_url: String,
    pub execution_rpc_url: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Deserialize)]
pub enum Network {
    Mainnet,
    Goerli,
}

impl Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Network::Mainnet => f.write_str("Mainnet"),
            Network::Goerli => f.write_str("Goerli"),
        }
    }
}

#[derive(Debug, Error)]
#[error("Bad network")]
pub struct BadNetwork;

impl FromStr for Network {
    type Err = BadNetwork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Mainnet" | "mainnet" => Ok(Network::Mainnet),
            "Goerli" | "goerli" => Ok(Network::Goerli),
            _ => Err(BadNetwork),
        }
    }
}
