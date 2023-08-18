use candid::Nat;
use contracts_abi::erc20::BalanceOfCall;
use ethers_core::abi::AbiEncode;
use interface::EstimateGasRequest;

mod test_canister;

use crate::test_canister::{call, setup_ethereum_canister};

#[test]
fn get_block_number() {
    let canister = setup_ethereum_canister();

    let block_num: (Nat,) = call!(canister, "get_block_number").unwrap();
    assert!(block_num.0 > 17880732u128);
}

#[test]
fn get_gas_price() {
    let canister = setup_ethereum_canister();

    let gas: (Nat,) = call!(canister, "get_gas_price").unwrap();
    assert_ne!(gas.0, 0u128);
}

#[test]
fn estimate_gas() {
    let canister = setup_ethereum_canister();

    let erc20_balance_of = BalanceOfCall {
        account: "0xF977814e90dA44bFA03b6295A0616a897441aceC"
            .parse()
            .unwrap(),
    };
    let request = EstimateGasRequest {
        from: None,
        to: "0xdAC17F958D2ee523a2206206994597C13D831ec7" // usdt
            .parse()
            .unwrap(),
        gas_limit: None,
        gas_price: None,
        value: None,
        data: Some(erc20_balance_of.encode()),
    };

    let gas: (Nat,) = call!(canister, "estimate_gas", request).unwrap();
    assert_ne!(gas.0, 0u128);
}

mod erc20 {
    use interface::{Erc20BalanceOfRequest, U256};

    use super::*;

    #[test]
    fn balance_of() {
        let canister = setup_ethereum_canister();

        let request = Erc20BalanceOfRequest {
            contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7" // usdt
                .parse()
                .unwrap(),
            account: "0xF977814e90dA44bFA03b6295A0616a897441aceC"
                .parse()
                .unwrap(),
        };
        let _: (U256,) = call!(canister, "erc20_balance_of", request).unwrap();
    }
}

mod erc721 {
    use interface::{Address, Erc721OwnerOfRequest};

    use super::*;

    #[test]
    fn owner_of() {
        let canister = setup_ethereum_canister();

        let request = Erc721OwnerOfRequest {
            contract: "0x5Af0D9827E0c53E4799BB226655A1de152A425a5" // milady
                .parse()
                .unwrap(),
            token_id: 7773_u32.into(),
        };

        let _: (Address,) = call!(canister, "erc721_owner_of", request).unwrap();
    }
}
