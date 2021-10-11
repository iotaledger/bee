// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bs58::{decode, encode};
use crypto::{
    hashes::sha::SHA256,
    signatures::ed25519::{PublicKey, PUBLIC_KEY_LENGTH},
};
use libp2p_core::multiaddr::Multiaddr;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize, Serializer,
};

use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    fmt,
    hash::Hash,
    ops::{Range, RangeInclusive},
    str::FromStr,
};

const AUTOPEERING_MULTIADDR_PROTOCOL_NAME: &str = "autopeering";
const PUBKEY_BASE58_SIZE_RANGE: RangeInclusive<usize> = 42..=44;

/// Go-libp2p allows Hornet to introduce a custom autopeering [`Protocol`]. In rust-libp2p we unfortunately can't do
/// that, so what we'll do is to introduce a wrapper type, which understands Hornet's custom multiaddr, and internally
/// stores the address part and the key part separatedly. The details are abstracted away and the behavior identical
/// to a standard libp2p multiaddress.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct AutopeeringMultiaddr {
    host_address: Multiaddr,
    public_key: PublicKey,
}

impl AutopeeringMultiaddr {
    pub fn host_address(&self) -> &Multiaddr {
        &self.host_address
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

#[derive(Debug)]
pub enum Error {
    /// Returned, if the host address part wasn't a valid multi address.
    InvalidHostAddressPart,
    /// Returned, if the public key part wasn't a base58 encoded ed25519 public key.
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
            self.host_address.to_string(),
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
            self.host_address.to_string(),
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

        let address = parts[0].parse().map_err(|_| Error::InvalidHostAddressPart)?;
        let public_key = from_base58_to_pubkey(parts[1]);

        Ok(Self {
            host_address: address,
            public_key,
        })
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

pub(crate) fn from_pubkey_to_base58(pub_key: &PublicKey) -> String {
    bs58::encode(pub_key.to_bytes()).into_string()
}

pub(crate) fn from_base58_to_pubkey(base58_pubkey: impl AsRef<str>) -> PublicKey {
    assert!(PUBKEY_BASE58_SIZE_RANGE.contains(&base58_pubkey.as_ref().len()));

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
    fn parse_sample_autopeering_multiaddr() {
        let bs58_pubkey = "HmKTkSd9F6nnERBvVbr55FvL1hM5WfcLvsc9bc3hWxWc";
        let autopeering_multiaddr = format!("/ip4/127.0.0.1/udp/14626/autopeering/{}", bs58_pubkey);

        let _: AutopeeringMultiaddr = autopeering_multiaddr
            .parse()
            .expect("parsing autopeering multiaddr failed");
    }

    #[test]
    fn parse_entrynode_multiaddrs() {
        let _: AutopeeringMultiaddr =
            "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM"
                .parse()
                .unwrap();
        let _: AutopeeringMultiaddr = "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb".parse().unwrap();
        let _: AutopeeringMultiaddr = "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2".parse().unwrap();
        let _: AutopeeringMultiaddr =
            "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
                .parse()
                .unwrap();
    }
}
