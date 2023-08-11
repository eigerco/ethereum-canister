use candid::Nat;

mod test_canister;

use crate::test_canister::{call, TestCanister};

#[test]
fn get_block_number() {
    let canister = TestCanister::setup_ethereum_canister();

    let block_num: (Nat,) = call!(canister, "get_block_number").unwrap();
    assert!(block_num.0 > 17880732u128);
}

mod erc20 {
    use super::*;

    #[test]
    fn balance_of() {
        let canister = TestCanister::setup_ethereum_canister();

        let request = interface::Erc20OwnerOfRequest {
            contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7" // usdt
                .parse()
                .unwrap(),
            account: "0xF977814e90dA44bFA03b6295A0616a897441aceC"
                .parse()
                .unwrap(),
        };
        let _: (Nat,) = call!(canister, "erc20_balance_of", request).unwrap();
    }
}

mod erc721 {
    use ethers_core::types::Address;

    use super::*;

    #[test]
    fn owner_of() {
        let canister = TestCanister::setup_ethereum_canister();

        let request = interface::Erc721OwnerOfRequest {
            contract: "0x5Af0D9827E0c53E4799BB226655A1de152A425a5" // milady
                .parse()
                .unwrap(),
            token_id: 7773_u32.into(),
        };

        let owner: (String,) = call!(canister, "erc721_owner_of", request).unwrap();
        owner.0.parse::<Address>().unwrap();
    }
}
