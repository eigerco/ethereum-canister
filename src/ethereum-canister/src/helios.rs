use std::cell::RefCell;
use std::rc::Rc;
use std::str;

use ethers_contract::EthCall;
use ethers_core::abi::{AbiDecode, AbiEncode};
use ethers_core::types::Address;
use eyre::{bail, eyre, Result, WrapErr};
use helios_client::database::ConfigDB;
use helios_client::{Client, ClientBuilder};
use helios_common::http;
use helios_common::types::BlockTag;
use helios_config::Network as HeliosNetwork;
use helios_execution::types::CallOpts;
use interface::Network;
use serde_json::Value;

thread_local! {
    static HELIOS: RefCell<Option<Rc<Client<ConfigDB>>>> = RefCell::new(None);
}

pub(crate) fn try_client() -> Option<Rc<Client<ConfigDB>>> {
    HELIOS.with(|helios| helios.borrow().clone())
}

pub(crate) fn client() -> Rc<Client<ConfigDB>> {
    try_client().expect("Client not started")
}

pub(crate) async fn start_client(
    network: Network,
    consensus_rpc_url: &str,
    execution_rpc_url: &str,
    checkpoint: Option<&str>,
) -> Result<()> {
    if HELIOS.with(|helios| helios.borrow().is_some()) {
        bail!("Client already started");
    }

    let network = match network {
        Network::Mainnet => HeliosNetwork::MAINNET,
        Network::Goerli => HeliosNetwork::GOERLI,
    };

    let checkpoint = if let Some(checkpoint) = checkpoint {
        checkpoint.to_owned()
    } else {
        fetch_latest_checkpoint(consensus_rpc_url)
            .await
            .wrap_err("Fetching latest checkpoint failed")?
    };

    let mut client: Client<ConfigDB> = ClientBuilder::new()
        .network(network)
        .consensus_rpc(consensus_rpc_url)
        .execution_rpc(execution_rpc_url)
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

pub(crate) async fn get_last_checkpoint() -> Option<String> {
    match try_client() {
        Some(client) => client.get_last_checkpoint().await,
        None => None,
    }
}

pub(crate) async fn shutdown() {
    if let Some(client) = try_client() {
        client.shutdown().await;
    }

    HELIOS.with(|helios| helios.borrow_mut().take());
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

async fn fetch_latest_checkpoint(consensus_rpc_url: &str) -> Result<String> {
    let checkpoint_url = format!("{consensus_rpc_url}/eth/v1/beacon/headers/finalized");
    let header_resp = http::get(&checkpoint_url)
        .await
        .wrap_err("Finalized header request failed")?;
    let body = str::from_utf8(&header_resp.body).wrap_err("Non utf-8 response")?;
    let header: Value = serde_json::from_str(body).wrap_err("Reading json response failed")?;
    let checkpoint = header
        .pointer("/data/root")
        .and_then(Value::as_str)
        .ok_or_else(|| eyre!("No root found in response: {body}"))?;
    Ok(checkpoint.to_owned())
}
