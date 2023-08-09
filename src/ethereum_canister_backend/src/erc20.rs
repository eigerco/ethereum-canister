use contracts_abi::erc20::*;
use ethers_core::abi::{AbiDecode, AbiEncode};
use ethers_core::types::{Address, U256};
use eyre::Result;
use helios_common::types::BlockTag;
use helios_execution::types::CallOpts;

use crate::global_client;

pub(crate) async fn balance_of(erc20_token: Address, wallet: Address) -> Result<U256> {
    let balance_of = BalanceOfCall { account: wallet };

    let opts = CallOpts {
        from: None,
        to: Some(erc20_token),
        gas: None,
        gas_price: None,
        value: None,
        data: Some(balance_of.encode()),
    };

    let bytes = global_client().call(&opts, BlockTag::Latest).await?;
    let ret = BalanceOfReturn::decode(bytes)?;

    Ok(ret.0)
}
