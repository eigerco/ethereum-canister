use std::str::FromStr;

use candid::types::{Compound, Serializer, Type};
use candid::CandidType;
use ethers_core::types::Address as EthersAddress;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(EthersAddress);

impl CandidType for Address {
    fn _ty() -> Type {
        <String as CandidType>::ty()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:?}", &self.0);
        let mut ser = serializer.serialize_struct()?;
        Compound::serialize_element(&mut ser, &s)?;
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let addr = s
            .parse::<EthersAddress>()
            .map_err(serde::de::Error::custom)?;
        Ok(Address(addr))
    }
}

impl From<EthersAddress> for Address {
    fn from(value: EthersAddress) -> Self {
        Address(value)
    }
}

impl From<Address> for EthersAddress {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl FromStr for Address {
    type Err = <EthersAddress as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr: EthersAddress = s.parse()?;
        Ok(Self(addr))
    }
}
