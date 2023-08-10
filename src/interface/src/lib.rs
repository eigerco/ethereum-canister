use std::str::FromStr;

use candid::types::{Compound, Serializer, Type};
use candid::{CandidType, Nat};
use ethers_core::types::{Address as EthersAddress, U256 as EthersU256};
use num_bigint::BigUint;
use serde::{Deserialize, Deserializer};
use thiserror::Error;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U256(EthersU256);

impl CandidType for U256 {
    fn _ty() -> Type {
        Nat::ty()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let nat: Nat = (*self).into();
        let mut ser = serializer.serialize_struct()?;
        Compound::serialize_element(&mut ser, &nat)?;
        Ok(())
    }
}

impl<'de> Deserialize<'de> for U256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nat = Nat::deserialize(deserializer)?;
        let num = nat.try_into().map_err(serde::de::Error::custom)?;
        Ok(num)
    }
}

impl From<EthersU256> for U256 {
    fn from(value: EthersU256) -> Self {
        U256(value)
    }
}

impl From<U256> for EthersU256 {
    fn from(value: U256) -> Self {
        value.0
    }
}

#[derive(Debug, Error)]
#[error("value bigger than u256")]
pub struct U256ConvertError;

impl TryFrom<Nat> for U256 {
    type Error = U256ConvertError;

    fn try_from(value: Nat) -> Result<Self, Self::Error> {
        let bytes = value.0.to_bytes_le();

        if bytes.len() > 32 {
            return Err(U256ConvertError);
        }

        let num = EthersU256::from_little_endian(&bytes);

        Ok(U256(num))
    }
}

impl From<U256> for Nat {
    fn from(value: U256) -> Self {
        let mut bytes = [0u8; 32];
        value.0.to_little_endian(&mut bytes);
        Nat(BigUint::from_bytes_le(&bytes))
    }
}

macro_rules! impl_u256_from_primitives {
    ($($typ:ty),+) => {
        $(
            impl ::core::convert::From<$typ> for crate::U256 {
                fn from(value: $typ) -> Self {
                    Self(value.into())
                }
            }
        )+
    };
}

macro_rules! impl_u256_try_from_primitives {
    ($($typ:ty),+) => {
        $(
            impl ::core::convert::TryFrom<$typ> for crate::U256 {
                type Error = <::ethers_core::types::U256 as TryFrom<$typ>>::Error;

                fn try_from(value: $typ) -> ::core::result::Result<Self, Self::Error> {
                    Ok(Self(value.try_into()?))
                }
            }
        )+
    };
}

impl_u256_from_primitives!(u8, u16, u32, u64, u128, usize);
impl_u256_try_from_primitives!(i8, i16, i32, i64, i128, isize);

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
