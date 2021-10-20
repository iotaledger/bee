// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::signatures::ed25519::{PublicKey, PUBLIC_KEY_LENGTH};
use libp2p_core::multiaddr::{Multiaddr, Protocol};
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize, Serializer,
};
use tokio::net::{lookup_host, ToSocketAddrs};

use std::{
    fmt,
    hash::Hash,
    net::{IpAddr, SocketAddr},
    ops::RangeInclusive,
    str::FromStr,
};

const AUTOPEERING_MULTIADDR_PROTOCOL_NAME: &str = "autopeering";
const PUBKEY_BASE58_SIZE_RANGE: RangeInclusive<usize> = 42..=44;

/// Go-libp2p allows Hornet to introduce a custom autopeering [`Protocol`]. In rust-libp2p we unfortunately can't do
/// that, so what we'll do is to introduce a wrapper type, which understands Hornet's custom multiaddr, and internally
/// stores the address part and the key part separatedly. The details are abstracted away and the behavior identical
/// to a standard libp2p multiaddress.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct AutopeeringMultiaddr {
    address: Multiaddr,
    public_key: PublicKey,
    resolved_addrs: Vec<SocketAddr>,
}

impl AutopeeringMultiaddr {
    /// Returns the address part.
    pub fn address(&self) -> &Multiaddr {
        &self.address
    }

    /// Returns the corresponding [`SocketAddr`] iff it contains an IPv4 or IPv6 address.
    ///
    /// Note: If the [`Multiaddr`] contains a DNS address, then `None` will be returned. In that case you
    /// should call `resolve_dns` and then `resolved_addrs` to get the corresponding [`SocketAddr`]s.
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        let mut multiaddr_iter = self.address().iter();

        let ip_addr = match multiaddr_iter.next().expect("error extracting ip address") {
            Protocol::Ip4(ip4_addr) => IpAddr::V4(ip4_addr),
            Protocol::Ip6(ip6_addr) => IpAddr::V6(ip6_addr),
            Protocol::Dns(_) => return None,
            _ => panic!("invalid multiaddr"),
        };

        let port = match multiaddr_iter.next().expect("error extracting port") {
            Protocol::Udp(port) => port,
            _ => panic!("invalid autopeering multiaddr"),
        };

        Some(SocketAddr::new(ip_addr, port))
    }

    /// Returns the [`PublicKey`].
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the resolved [`SocketAddr`]s determined by the last call to `resolve_dns`. If that method
    /// was never called before, the returned slice will be empty.
    pub fn resolved_addrs(&self) -> &[SocketAddr] {
        &self.resolved_addrs[..]
    }

    /// Performs DNS resolution if this multiaddr contains a DNS address.
    pub async fn resolve_dns(&mut self) -> bool {
        self.resolved_addrs.clear();

        let mut address_iter = self.address.iter();

        let dns = match address_iter.next().expect("error extracting ip address") {
            Protocol::Dns(dns) => dns,
            _ => return false,
        };

        let port = match address_iter.next().expect("error extracting port") {
            Protocol::Udp(port) => port,
            _ => panic!("invalid autopeering multiaddr"),
        };

        let host = format!("{}:{}", dns.as_ref(), port);

        if let Ok(socket_addrs) = lookup_host(host).await {
            self.resolved_addrs.extend(socket_addrs);
            true
        } else {
            false
        }
    }
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
        let s = format_autopeering_multiaddr(&self.address, &self.public_key);
        serializer.serialize_str(&s)
    }
}

impl fmt::Debug for AutopeeringMultiaddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AutopeeringMultiaddr")
            .field("host_multiaddr", &self.address)
            .field("public_key", &from_pubkey_to_base58(&self.public_key))
            .finish()
    }
}

impl fmt::Display for AutopeeringMultiaddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_autopeering_multiaddr(&self.address, &self.public_key))
    }
}

fn format_autopeering_multiaddr(host_multiaddr: &Multiaddr, public_key: &PublicKey) -> String {
    format!(
        "{}/{}/{}",
        host_multiaddr,
        AUTOPEERING_MULTIADDR_PROTOCOL_NAME,
        from_pubkey_to_base58(public_key),
    )
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
        let resolved_addrs = Vec::new();

        Ok(Self {
            address,
            public_key,
            resolved_addrs,
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

#[derive(Debug)]
pub enum Error {
    /// Returned, if the host address part wasn't a valid multi address.
    InvalidHostAddressPart,
    /// Returned, if the public key part wasn't a base58 encoded ed25519 public key.
    InvalidPubKeyPart,
    /// Returned, if it's not a valid autopeering multi address.
    InvalidAutopeeringMultiaddr,
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
