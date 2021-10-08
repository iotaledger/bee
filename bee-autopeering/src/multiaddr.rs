// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bs58::{decode, encode};
use crypto::hashes::sha::SHA256;
use crypto::signatures::ed25519::{PublicKey, PUBLIC_KEY_LENGTH};
use libp2p_core::multiaddr::Multiaddr;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize, Serializer,
};

use std::convert::TryInto;
use std::hash::Hash;
use std::{borrow::Cow, convert::TryFrom, fmt, str::FromStr};

/// Go-libp2p allows Hornet to introduce a custom autopeering [`Protocol`]. In rust-libp2p we unfortunately can't do
/// that, so what we'll do is to introduce a wrapper type, which understands Hornet's custom multiaddr, and internally
/// stores the address part and the key part separatedly. The details are abstracted away and the behavior identical
/// to a standard libp2p multiaddress.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct AutopeeringMultiaddr {
    address: Multiaddr,
    public_key: PublicKey,
}

impl AutopeeringMultiaddr {
    pub fn address(&self) -> &Multiaddr {
        &self.address
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

const AUTOPEERING_MULTIADDR_PROTOCOL_NAME: &str = "autopeering";

#[derive(Debug)]
pub enum Error {
    /// Returned, if the dial part wasn't a valid multi address.
    InvalidAddressPart,
    /// Returned, if the key part wasn't a base58 encoded ed25519 public key.
    InvalidPubKeyPart,
    /// Returned, if it's not a valid autopeering multi address.
    InvalidAutopeeringMultiaddr,
}

impl<'de> Deserialize<'de> for AutopeeringMultiaddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(AutopeeringMultiaddrVisitor)
    }
}

impl Serialize for AutopeeringMultiaddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!(
            "{}/{}/{}",
            self.address.to_string(),
            AUTOPEERING_MULTIADDR_PROTOCOL_NAME,
            from_pubkey_to_base58(&self.public_key),
        );

        serializer.serialize_str(&s)
    }
}

impl fmt::Display for AutopeeringMultiaddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.address.to_string(),
            AUTOPEERING_MULTIADDR_PROTOCOL_NAME,
            from_pubkey_to_base58(&self.public_key),
        )
    }
}

impl FromStr for AutopeeringMultiaddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = s
            .find(AUTOPEERING_MULTIADDR_PROTOCOL_NAME)
            .ok_or(Error::InvalidAutopeeringMultiaddr)?;

        let parts = s
            .split_terminator(&format!("/{}/", AUTOPEERING_MULTIADDR_PROTOCOL_NAME))
            .collect::<Vec<&str>>();

        if parts.len() != 2 {
            return Err(Error::InvalidAutopeeringMultiaddr);
        }

        let address = parts[0].parse().map_err(|_| Error::InvalidAddressPart)?;
        let public_key = from_base58_to_pubkey(parts[1]);

        Ok(Self { address, public_key })
    }
}

struct AutopeeringMultiaddrVisitor;

impl<'de> Visitor<'de> for AutopeeringMultiaddrVisitor {
    type Value = AutopeeringMultiaddr;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an autopeering multiaddr")
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.parse().expect("failed to parse autopeering multiaddr"))
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.parse().expect("failed to parse autopeering multiaddr"))
    }
}

fn from_pubkey_to_base58(pub_key: &PublicKey) -> String {
    bs58::encode(pub_key.to_bytes()).into_string()
}

fn from_base58_to_pubkey(base58_pubkey: impl AsRef<str>) -> PublicKey {
    let mut bytes = [0u8; PUBLIC_KEY_LENGTH];
    bs58::decode(base58_pubkey.as_ref())
        .into(&mut bytes)
        .expect("error decoding base58 pubkey");
    PublicKey::try_from_bytes(bytes).expect("error restoring public key from bytes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    // NOTE: example taken from Hornet!
    //
    // ```go
    // base58PubKey := "HmKTkSd9F6nnERBvVbr55FvL1hM5WfcLvsc9bc3hWxWc"
    // smpl := fmt.Sprintf("/ip4/127.0.0.1/udp/14626/autopeering/%s", base58PubKey)
    // ma, err := multiaddr.NewMultiaddr(smpl)
    // ```

    #[test]
    fn convert_between_base58_and_pubkey() {
        let base58_pubkey = "4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM";
        let pubkey = from_base58_to_pubkey(base58_pubkey);

        assert_eq!(base58_pubkey, from_pubkey_to_base58(&pubkey))
    }

    #[test]
    fn parse_autopeering_multiaddr() {
        let bs58_pubkey = "HmKTkSd9F6nnERBvVbr55FvL1hM5WfcLvsc9bc3hWxWc";
        let autopeering_multiaddr = format!("/ip4/127.0.0.1/udp/14626/autopeering/{}", bs58_pubkey);

        let _: AutopeeringMultiaddr = autopeering_multiaddr
            .parse()
            .expect("parsing autopeering multiaddr failed");
    }
}
