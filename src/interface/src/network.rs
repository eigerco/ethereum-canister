use std::fmt::{self, Display};
use std::str::FromStr;

use candid::CandidType;
use serde::Deserialize;
use thiserror::Error;

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
