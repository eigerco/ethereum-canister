use std::cell::RefCell;
use std::rc::Rc;

use ethers_contract::EthCall;
use ethers_core::abi::{AbiDecode, AbiEncode};
use ethers_core::types::Address;
use eyre::{bail, Result, WrapErr};
use helios_client::database::ConfigDB;
use helios_client::{Client, ClientBuilder};
use helios_common::types::BlockTag;
use helios_config::Network;
use helios_execution::types::CallOpts;

thread_local! {
    static HELIOS: RefCell<Option<Rc<Client<ConfigDB>>>> = RefCell::new(None);
}

pub(crate) fn client() -> Rc<Client<ConfigDB>> {
    HELIOS
        .with(|helios| helios.borrow().clone())
        .expect("Client not started")
}

pub(crate) async fn start(
    consensus_rpc_url: &str,
    execution_rpc_url: &str,
    checkpoint: &str,
) -> Result<()> {
    if HELIOS.with(|helios| helios.borrow().is_some()) {
        bail!("Client already started");
    }

    let mut client: Client<ConfigDB> = ClientBuilder::new()
        .network(Network::MAINNET)
        .consensus_rpc(&consensus_rpc_url)
        .execution_rpc(&execution_rpc_url)
        .checkpoint(&checkpoint)
        .load_external_fallback()
        .build()
        .wrap_err("Client setup failed")?;

    client
        .start()
        .await
        .wrap_err("Failed to start the client")?;

    HELIOS.with(|helios| *helios.borrow_mut() = Some(Rc::new(client)));

    Ok(())
}

pub(crate) async fn call<T, R>(contract: Address, call_data: T) -> Result<R>
where
    T: EthCall + AbiEncode,
    R: AbiDecode,
{
    let opts = CallOpts {
        from: None,
        to: Some(contract),
        gas: None,
        gas_price: None,
        value: None,
        data: Some(call_data.encode()),
    };

    let bytes = client().call(&opts, BlockTag::Latest).await?;
    let ret = R::decode(bytes)?;

    Ok(ret)
}
