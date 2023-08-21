# Ethereum Canister

The Ethereum canister offers a secure and trustless way to access Ethereum blockchain data within the ICP ecosystem.
Behind the scenes, it leverages the [`helios`](https://github.com/a16z/helios) light Ethereum client.
This client is equipped with the capability to validate the authenticity of fetched data.

## Running and using the Ethereum canister

```bash
dfx start --clean --background --artificial-delay 100
# deploy
dfx deploy
# start it
dfx canister call ethereum_canister setup 'record {
    network = variant { Mainnet };
    execution_rpc_url = "https://ethereum.publicnode.com";
    consensus_rpc_url = "https://www.lightclientdata.org";
}'
# utilize it
dfx canister call ethereum_canister erc20_balance_of 'record {
    contract = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    account = "0xF977814e90dA44bFA03b6295A0616a897441aceC";
}'
(2_100_000_000_000_000 : nat) # canister's output
```

## Running end-to-end canister tests

```bash
dfx start --clean --background --artificial-delay 100

cargo test --target x86_64-unknown-linux-gnu
# or using nextest
cargo nextest run --target x86_64-unknown-linux-gnu
```

## Design

Ethereum canister is a thin wrapper around the `helios` light client.
The client operates as global state and conducts periodic synchronization with the Ethereum consensus node in the background.
Canister exposes a trustless RPC API of the execution node from the `helios` for other canisters.

### Canister API

You can find the API definition for the Ethereum canister in the [`candid.did` file](src/ethereum_canister/candid.did).
Additionally all the api types for Rust canisters are in the [`interface`](src/interface/src/lib.rs) crate
(will probably use a rename in the future) that doesn't depend on `helios` directly but on a lighter `ethers-core` for that matter.

The canister mostly just exposes functions from the underlying helios client. Most of the users familiar with
Ethereum will likely be familiar with them too so there is no point in trying to be innovative there. Those usually just
have the same name, take the same arguments, and return the same types (or their `Candid` counterparts).

Functions are categorized as update or query based on whether they involve making an RPC call to the execution node or not.
Examples of queries can be `get_block_number` or `get_gas_price`, which just take information from already synchronized blocks.
Examples of updates are calling the function of a contract or estimating the gas for a transaction as it requires fetching
the addresses and other data.

For the smart contract standards like the ERC20 or ERC721 the exposed API's are in form `${standard_name}_${function_name}`
eg. `erc20_balance_of`. The parameters to those functions are Candid's equivalents for the parameters from the contract's standard ABI.
It is on the ethereum canister to properly encode them before making a `call`.

Presently, the setup function is the only exception to the aforementioned categorization. It is responsible for configuring and
initiating the helios client. It is required to be called before any other function, otherwise, the called function will return an error.
It takes urls to the consensus node and execution node the client will connect to, as well as the type of 
network it should operate on and an optional weak subjectivity checkpoint, a trusted hash of a block that nodes agree on. If not
provided, the last checkpoint from provided consensus node will be taken. Please note that providing a checkpoint that is too old
can result in much more https outcalls and computations needed to reach the synchronization,
in some cases exceeding the limits of an update call.

### Synchronization

The background loop utilizes `ic_cdk_timers::set_timer_interval` and operates at 12-second intervals, mirroring the Ethereum slot time.
This is the only place where the running helios client is ever mutated and the time of locking for the updates was reduced to the
required minimum which should be unnoticeable.

### Upgrades

The canister stores its configuration and the latest checkpoint it has reached in stable memory. When upgrading the canister
it should restart the helios client itself with the previous configuration and the checkpoint that was already trusted. The last
64 blocks will be re-fetched.

### Error handling

Exploring error handling strategies for inter-canister calls reveals two options: returning Results or triggering panics upon errors.
The panicking way was chosen, as the `Result`s seemed less ergonomic for potential developers and they feel a bit inconsistent
as `CallResult` already has `CanisterError` and `CanisterReject` variants. Also going with the panics and having the state reverted
felt more familiar with the smart contracts on other blockchains.

## Implementation notes

### Helios

The foundation of this canister. The ethereum canister depends on the `client`, `consensus`, `execution`, and `common` helios crates.
To be able to use a helios on the ICP it had [to be forked](https://github.com/eigerco/helios). The fork introduced many
changes to the internals of helios thus making itself not-upstreamable in its current form. Probably it is possible to upstream
some of those changes and maybe even make it compatible with the ICP ecosystem but this is considered to be not a trivial task
with many possible ways of achieving the goals that should be considered and brainstormed.
The changes were made in a manner that helios still works perfectly fine for native, but when targeting wasm it only supports
the ICP and no longer the browsers.

The changes include:
- updating the `helios` and making it compatible with the rust `stable` toolchain
- getting rid of any occurrences of `wasm-bindgen`, `glue-timers`, and other browser-related wasm dependencies
- getting rid of `tokio`-related stuff that is incompatible with wasm
- using `ethers-core` where possible and removing `ethers-providers` entirely
- replace `reqwest` with https outcalls when targeting wasm
- update the `revm` to v3 and introduce a way to fetch missing slots outside in an async manner
- change the updating logic to only lock for a short period and add proper shutdown and cleanup

For the complete list of changes see this [comparison](https://github.com/a16z/helios/compare/master...eigerco:helios:master).

### Ethers

The `ethers` crate is the building block of helios and the home for most of the types from the Ethereum ecosystem. It also
provides the utilities to generate types and encoding logic from the smart contract's ABI. It would be really useful to have this
crate working for the ICP ecosystem completely nonetheless this is considered a nontrivial task. The main problem is that it
has already good support for browser-side wasm and some of its subcrates utilize not supported crates heavily. The main pain
point we've encountered was the `ethers-providers` crate. The best way to approach this would be to support ICP in crates
like [`instant`](https://github.com/sebcrozet/instant) and [`futures-timer`](https://github.com/async-rs/futures-timer) or even
[`gloo-timers`](https://github.com/rustwasm/gloo), guard the http implementation behind the feature flag and add a way for a user
to add a custom provider.

There was one change made to the ethers, allowing the use of `ethers-contract` without the need to depend on `ethers-providers`
and it was [already upstreamed](https://github.com/gakonst/ethers-rs/pull/2536).

## Cost analysis

The measurements presented below were acquired on a local dfx network setup. These measurements encompass supplementary
logs and logic integrated into the canister's code and its dependencies. They are not precise and shouldn't be taken as truth,
rather just to shed some light on the potential costs.

|                             | call cost [cycles] | payments [cycles] |  instructions | https outcalls |
|:----------------------------|-------------------:|------------------:|--------------:|---------------:|
| setup                       |    122,579,275,431 |   122,553,254,989 | 1,149,129,243 |             74 |
| advance                     |     14,126,362,419 |    12,794,806,467 |   839,856,523 |              8 |
| get_gas_price               |            105,594 |                 0 |         5,605 |              0 |
| get_block_number            |            101,230 |                 0 |         2,336 |              0 |
| erc20_balance_of **[SHIB]** |     14,438,775,194 |    14,438,169,162 |    10,996,872 |              9 |
| erc20_balance_of **[BNB]**  |     14,439,107,417 |    14,438,470,723 |    12,031,945 |              9 |
| erc20_balance_of **[USDT]** |     17,655,037,455 |    17,654,021,703 |    24,259,653 |             11 |
| erc721_owner_of **[ENS]**   |     17,654,474,853 |    17,653,498,044 |    23,011,079 |             11 |
| erc721_owner_of **[DREAD]** |     17,656,442,523 |    17,655,209,030 |    31,469,631 |             11 |
| erc721_owner_of **[MIL]**   |     20,866,407,305 |    20,865,374,497 |    25,107,423 |             13 |

All the functions were measured with the advancing (sync loop) disabled except the advance itself.

All the functions were measured with 5 repetitions and the maximum acquired value was presented in the table.

The `setup` function was measured with fetching the latest checkpoint from the consensus node.

The 'call cost' was measured using the `dfx canister status` command before and after executing the case.

The 'payments' was measured using the difference between `ic_cdk::api::canister_balance` invoked at the beginning
and the end of the function. It should reflect the amount spent for https outcalls.

The 'instructions' were measured using the difference between `ic_cdk::api::performance_counter` invoked at the
beginning and the end of the function. It should reflect the number of wasm instructions executed during the call.

The 'https outcalls' was measured by counting the calls to the `http::get` and `http::post` functions.

## Next steps

### Optimization ideas

Https outcalls are the biggest factor of the costs and it should be the primary focus when optimizing.

Among the methods, the setup call incurs the highest cost due to its requirement to initialize everything for the first time
and retrieve the last 64 blocks. Unfortunately, it uses mostly the beacon API which is a restful http api, not a json-rpc like
the execution API, so requests cannot be batched easily. And even then, a single block can be bigger than 1MB so fetching two
at the time could exceed the limit [of 2MB](https://internetcomputer.org/docs/current/developer-docs/integrations/https-outcalls/https-outcalls-how-it-works)
for the https outcall.

However, it seems to be possible to heavily optimize all the execution RPC calls. Here is a sample log from the smart contract call.

<details>

<summary>Log - erc721_owner_of <b>[MIL]</b></summary>

```
common::http::icp] POST https://ethereum.publicnode.com {"id":0,"jsonrpc":"2.0","method":"eth_createAccessList","params":[{"type":"0x02","to":"0x5af0d9827e0c53e4799bb226655a1de152a425a5","gas":"0x5f5e100","data":"0x6352211e0000000000000000000000000000000000000000000000000000000000001e5d","accessList":[],"maxPriorityFeePerGas":"0x0","maxFeePerGas":"0x0"},"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":1,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa","0x0000000000000000000000000000000000000000000000000000000000000002","0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb9789"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":2,"jsonrpc":"2.0","method":"eth_getProof","params":["0x0000000000000000000000000000000000000000",[],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":3,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",[],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":4,"jsonrpc":"2.0","method":"eth_getProof","params":["0x388c818ca8b9251b393131c08a736a67ccb19297",[],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":5,"jsonrpc":"2.0","method":"eth_getCode","params":["0x388c818ca8b9251b393131c08a736a67ccb19297","0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":6,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":7,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114

execution::evm] Missing slots: MissingSlots { address: 0x5af0d9827e0c53e4799bb226655a1de152a425a5, slots: [0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa] }
common::http::icp] POST https://ethereum.publicnode.com {"id":8,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":9,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=2

execution::evm] Missing slots: MissingSlots { address: 0x5af0d9827e0c53e4799bb226655a1de152a425a5, slots: [0x0000000000000000000000000000000000000000000000000000000000000002] }
common::http::icp] POST https://ethereum.publicnode.com {"id":10,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x0000000000000000000000000000000000000000000000000000000000000002","0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":11,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=2
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=29102676481673041902632991033461445430619272659676223336789171408008386418569

execution::evm] Missing slots: MissingSlots { address: 0x5af0d9827e0c53e4799bb226655a1de152a425a5, slots: [0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb9789] }
common::http::icp] POST https://ethereum.publicnode.com {"id":12,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb9789","0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa","0x0000000000000000000000000000000000000000000000000000000000000002"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":13,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=2
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=29102676481673041902632991033461445430619272659676223336789171408008386418569
```

</details>

Firstly, optimizing `Evm::batch_fetch_accounts` could involve consolidating requests to fetch codes and proofs into a single operation after establishing the access list.
Also `ExecutionClient::fetch_account` could fetch the code and proof in one go. That would reduce the amount of https outcalls used in this specific case from 13 to 5.
Another avenue to explore is determining the additional required slots for EVM execution and pre-fetching them. Alternatively, seeking a method to acquire a
comprehensive list of missing addresses from revm in a single instance—without resimulation—could be pursued, albeit this might entail modifications in revm.
Depending on the outcomes, this might lead to a potential reduction in the number of calls from 5 down to 3 or possibly even 2.

If any of the urls provided as a configuration for the canister leads to the http redirections (eg. "https://www.lightclientdata.org" results in redirects), then all
the https outcalls will be repeated for the original url and then for the redirected on. Supporting redirections can be useful when self hosting nodes for this project,
so it would be good to optimize it in a way it first checks for the redirections and then storing the actual final url for the requests. The original url can then be kept
as a fallback url in case the redirected one is no longer valid.

### Canister 

The initial development was mainly focused on getting the `helios` and `ethers` to the point where they operate correctly in the ICP environment.
The next step would be to apply all the best practices for canisters implementation including things like controllers, metrics collection,
API design, handling payments, and guarding some methods from public usage.

Also exposing the full helios API and seeking community input sounds like a neat idea.
