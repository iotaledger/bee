// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::address::{Address, Ed25519Address};
use crypto::{
    hashes::{blake2b::Blake2b256, Digest}
};
use structopt::StructOpt;
use thiserror::Error;

const BECH32_HRP: &str = "iota";

#[derive(Clone, Debug, Error)]
pub enum ConvertError {
    #[error("invalid Bech32 address length")]
    InvalidAddressLength(),
    #[error("Invalid Address")]
    InvalidAddress(),
}

#[derive(Clone, Debug, StructOpt)]
pub enum ConvertTool {
    /// convert Bech32 address to hex encoding
    Bech32ToHex {  bech32: String },
    /// convert hex encoding to Bech32 address 
    HexToBech32 { hex: String},
    /// convert a hex encoded public key to Bech32 address
    HexPubkeyToBech32 { pubkey: String},
}

pub fn exec(tool: &ConvertTool) -> Result<(), ConvertError> {
    match tool {
        ConvertTool::Bech32ToHex { bech32 } => {
            let hex = bech32_to_hex(&bech32.as_str()).unwrap();
            println!("Your Hex encoded address is:\t{:?}", hex);
        }
        ConvertTool::HexToBech32 { hex } => {
            let bech32 = hex_to_bech32(&hex.as_str(), BECH32_HRP).unwrap();
            println!("Your Bech32 address is:\t{:?}", bech32);
        }
        ConvertTool::HexPubkeyToBech32 { pubkey } => {
            let bech32 = hex_public_key_to_bech32_address(&pubkey.as_str(), BECH32_HRP).unwrap();
            println!("Your Bech32 address is:\t{:?}", bech32);
        }
    }
    Ok(())
}

/// Transforms bech32 to hex
fn bech32_to_hex(bech32: &str) -> Result<String, ConvertError> {
    let address = Address::try_from_bech32(bech32).map_err(|_| ConvertError::InvalidAddress())?;
    let hex_string = match address {
        (_, Address::Ed25519(ed)) => ed.to_string(),
        (_, Address::Alias(alias)) => alias.to_string(),
        (_, Address::Nft(nft)) => nft.to_string(),
    };
    Ok(hex_string)
}

/// Transforms a hex encoded address to a bech32 encoded address
pub fn hex_to_bech32(hex: &str, bech32_hrp: &str) -> Result<String, ConvertError> {
    let address: Ed25519Address = hex.parse::<Ed25519Address>().map_err(|_| ConvertError::InvalidAddress())?;
    Ok(Address::Ed25519(address).to_bech32(bech32_hrp))
}

/// Transforms a hex encoded public key to a bech32 encoded address
pub fn hex_public_key_to_bech32_address(hex: &str, bech32_hrp: &str) -> Result<String, ConvertError> {
    let mut public_key = [0u8; Ed25519Address::LENGTH];
    hex::decode_to_slice(&hex, &mut public_key).map_err(|_| ConvertError::InvalidAddressLength())?;

    let address = Blake2b256::digest(&public_key)
        .try_into()
        .map_err(|_e| ConvertError::InvalidAddress())?;
    let address: Ed25519Address = Ed25519Address::new(address);
    Ok(Address::Ed25519(address).to_bech32(bech32_hrp))
}


#[cfg(test)]
mod bech32tests {
    use crate::tools::convert::*;
    // spec: https://github.com/iotaledger/tips/blob/main/tips/TIP-0011/tip-0011.md
    #[test]
    fn bech32tohex() {
        let bech32tohex = ConvertTool::Bech32ToHex{ bech32 : "iota1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6xqgyzyx".to_string()};
        exec(&bech32tohex); // output: "0xefdc112efe262b304bcf379b26c31bad029f616ee3ec4aa6345a366e4c9e43a3"
    }

    #[test]
    fn hextobech32() {
        let hextobech32 = ConvertTool::HexToBech32 { hex : "0xefdc112efe262b304bcf379b26c31bad029f616ee3ec4aa6345a366e4c9e43a3".to_string()};
        exec(&hextobech32).unwrap(); // output: "iota1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6xqgyzyx"
    }

    #[test]
    fn pubkeytohex() {
        let pubkeytohex = ConvertTool::HexPubkeyToBech32 { pubkey : "6f1581709bb7b1ef030d210db18e3b0ba1c776fba65d8cdaad05415142d189f8".to_string()};
        exec(&pubkeytohex);  // output: "iota1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6xqgyzyx"
    }
}
