use contracts_abi::erc721::*;
use ethers_core::types::{Address, U256};
use eyre::Result;

use crate::helios;

pub(crate) async fn owner_of(erc721_contract: Address, token_id: U256) -> Result<Address> {
    let ret: OwnerOfReturn = helios::call(erc721_contract, OwnerOfCall { token_id }).await?;
    Ok(ret.0)
}
