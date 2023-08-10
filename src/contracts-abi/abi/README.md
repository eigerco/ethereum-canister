## How to generate ABI from sol file

You can run the following command and the ABI will be printed to stdout.

```
solc --abi openzeppelin-contracts/contract/token/ERC20/ERC20.sol
```

Also use `jq` or any other tools to prettify json.


## How to get ABI from Etherscan

Navigate to the contract page, for example for WETH:

https://etherscan.io/token/0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2#code

Go to `Contract` tab and copy the ABI from `Contract ABI` section.
