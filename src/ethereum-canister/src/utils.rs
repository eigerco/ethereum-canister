use candid::Nat;
use ethers_core::types::U256;
use helios_execution::types::CallOpts;
use interface::EstimateGasRequest;
use num_bigint::BigUint;

pub(crate) trait ToNat {
    fn to_nat(&self) -> Nat;
}

impl ToNat for U256 {
    fn to_nat(&self) -> Nat {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        Nat(BigUint::from_bytes_le(&bytes))
    }
}

pub(crate) trait IntoCallOpts {
    fn into_call_opts(self) -> CallOpts;
}

impl IntoCallOpts for EstimateGasRequest {
    fn into_call_opts(self) -> CallOpts {
        CallOpts {
            from: self.from.map(Into::into),
            to: self.to.map(Into::into),
            gas: self.gas_limit.map(Into::into),
            gas_price: self.gas_price.map(Into::into),
            value: self.value.map(Into::into),
            data: self.data,
        }
    }
}
