pub use crate::inner::crypto_kitties as cryptokitties;
pub use crate::inner::crypto_punks as cryptopunks;
pub use crate::inner::erc_1155 as erc1155;
pub use crate::inner::erc_165 as erc165;
pub use crate::inner::erc_20 as erc20;
pub use crate::inner::erc_721 as erc721;
pub use crate::inner::weth;

// Workaround for removing re-exports that `abigen` adds
mod inner {
    use ethers_contract::abigen;

    abigen!(CryptoKitties, "src/contracts-abi/abi/cryptokitties.json");
    abigen!(CryptoPunks, "src/contracts-abi/abi/cryptopunks.json");
    abigen!(Erc1155, "src/contracts-abi/abi/erc1155.json");
    abigen!(Erc165, "src/contracts-abi/abi/erc165.json");
    abigen!(Erc20, "src/contracts-abi/abi/erc20.json");
    abigen!(Erc721, "src/contracts-abi/abi/erc721.json");
    abigen!(Weth, "src/contracts-abi/abi/weth.json");
}
