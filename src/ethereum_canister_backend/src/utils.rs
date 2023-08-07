use candid::Nat;
use ethers_core::types::U256;
use num_bigint::BigUint;

pub trait ToNat {
    fn to_nat(&self) -> Nat;
}

impl ToNat for U256 {
    fn to_nat(&self) -> Nat {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        Nat(BigUint::from_bytes_le(&bytes))
    }
}
