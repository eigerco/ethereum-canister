use contracts_abi::erc20::*;
use ethers_core::types::{Address, U256};
use eyre::Result;

use crate::helios;

pub(crate) async fn balance_of(erc20_contract: Address, account: Address) -> Result<U256> {
    let ret: BalanceOfReturn = helios::call(erc20_contract, BalanceOfCall { account }).await?;
    Ok(ret.0)
}
