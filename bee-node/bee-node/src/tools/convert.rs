// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::address::{Address, Ed25519Address};
use crypto::hashes::{blake2b::Blake2b256, Digest};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ConvertError {
    #[error("invalid Bech32 address length")]
    InvalidAddressLength(),
    #[error("Invalid Address")]
    InvalidAddress(),
}

#[derive(Clone, Debug, StructOpt)]
pub enum ConvertTool {
    /// Converts a Bech32 address to a hex encoded one.
    Bech32ToHex {
        #[structopt(long)]
        bech32: String,
    },
    /// Converts a hex encoded address to a Bech32 one.
    HexToBech32 {
        #[structopt(long)]
        hex: String,
        #[structopt(long)]
        hrp: String,
    },
    /// Converts a hex encoded public key to a Bech32 address.
    HexPubkeyToBech32 {
        #[structopt(long)]
        pubkey: String,
        #[structopt(long)]
        hrp: String,
    },
}

pub fn exec(tool: &ConvertTool) -> Result<(), ConvertError> {
    match tool {
        ConvertTool::Bech32ToHex { bech32 } => {
            let hex = bech32_to_hex(bech32.as_str());
            match hex {
                Ok(_) => println!("Your Hex encoded address is:\t{}", hex.unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        }
        ConvertTool::HexToBech32 { hex, hrp } => {
            let bech32 = hex_to_bech32(hex.as_str(), hrp.as_str());
            match bech32 {
                Ok(_) => println!("Your Bech32 address is:\t{:?}", bech32.unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        }
        ConvertTool::HexPubkeyToBech32 { pubkey, hrp } => {
            let bech32 = hex_public_key_to_bech32_address(pubkey.as_str(), hrp.as_str());
            match bech32 {
                Ok(_) => println!("Your Bech32 address is:\t{:?}", bech32.unwrap()),
                Err(e) => println!("Error: {}", e),
            }
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
fn hex_to_bech32(hex: &str, bech32_hrp: &str) -> Result<String, ConvertError> {
    let address: Ed25519Address = hex
        .parse::<Ed25519Address>()
        .map_err(|_| ConvertError::InvalidAddress())?;
    Ok(Address::Ed25519(address).to_bech32(bech32_hrp))
}

/// Transforms a hex encoded public key to a bech32 encoded address
fn hex_public_key_to_bech32_address(hex: &str, bech32_hrp: &str) -> Result<String, ConvertError> {
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
    fn bech32_to_hex() {
        let bech32_to_hex = ConvertTool::Bech32ToHex { 
            bech32: "iota1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6xqgyzyx".to_string()
        };
        // output: "0xefdc112efe262b304bcf379b26c31bad029f616ee3ec4aa6345a366e4c9e43a3";
        match exec(&bech32_to_hex){
            Err(e) => println!("{:?}", e),
            _ => ()
        };
    }

    #[test]
    fn hex_to_bech32() {
        let hex_to_bech32 = ConvertTool::HexToBech32 {
            hex: "0xefdc112efe262b304bcf379b26c31bad029f616ee3ec4aa6345a366e4c9e43a3".to_string(),
            hrp: "iota".to_string(),
        };
        // output: "iota1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6xqgyzyx";
        match exec(&hex_to_bech32){
            Err(e) => println!("{:?}", e),
            _ => ()
        };
    }

    #[test]
    fn public_key_to_hex() {
        let public_key_to_hex = ConvertTool::HexPubkeyToBech32 {
            pubkey: "6f1581709bb7b1ef030d210db18e3b0ba1c776fba65d8cdaad05415142d189f8".to_string(),
            hrp: "iota".to_string(),
        };
        // output:  "iota1qrhacyfwlcnzkvzteumekfkrrwks98mpdm37cj4xx3drvmjvnep6xqgyzyx";
        match exec(&public_key_to_hex){
            Err(e) => println!("{:?}", e),
            _ => ()
        };
    }
}
