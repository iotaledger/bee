// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::signatures::ed25519::{PublicKey, PUBLIC_KEY_LENGTH};
use libp2p_core::multiaddr::{Multiaddr, Protocol};
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize, Serializer,
};
use tokio::net::lookup_host;

use std::{
    fmt,
    hash::Hash,
    net::{IpAddr, SocketAddr},
    ops::RangeInclusive,
    str::FromStr,
};

const AUTOPEERING_MULTIADDR_PROTOCOL_NAME: &str = "autopeering";
const PUBKEY_BASE58_SIZE_RANGE: RangeInclusive<usize> = 42..=44;

/// The different supported kinds of addresses.
pub enum AddressKind {
    /// Static `IPv4` address.
    Ip4,
    /// Static `IPv6` address.
    Ip6,
    /// A domain name that needs to be resolved to an `IPv4`, `IPv6` address (or both) at runtime.
    Dns,
}

// Note:
// Go-libp2p allows Hornet to introduce a custom autopeering [`Protocol`]. In rust-libp2p we unfortunately can't do
// that, so what we'll do is to introduce a wrapper type, which understands Hornet's custom multiaddr, and internally
// stores the address part and the key part separatedly. The details are abstracted away and the behavior identical
// to a standard libp2p multiaddress.

/// Represents a special type of [`Multiaddr`] used to describe peers that particpate in autopeering, i.e. make
/// themselves discoverable by other peers.
///
/// Example:
///
/// "/dns/chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb"
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

    /// Returns the kind of this address.
    pub fn address_kind(&self) -> AddressKind {
        // Panic: `self` must always contain a valid address.
        let kind = self.address.iter().next().expect("invalid multiaddress");
        match kind {
            Protocol::Ip4(_) => AddressKind::Ip4,
            Protocol::Ip6(_) => AddressKind::Ip6,
            Protocol::Dns(_) => AddressKind::Dns,
            _ => panic!("unsupported address kind"),
        }
    }

    /// Returns the corresponding [`SocketAddr`] iff it contains an IPv4 or IPv6 address.
    ///
    /// Note: If the [`Multiaddr`] contains a DNS address, then `None` will be returned. In that case you
    /// should call `resolve_dns` and then `resolved_addrs` to get the corresponding [`SocketAddr`]s.
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        let mut multiaddr_iter = self.address().iter();

        // Panic: `self` must always contain a valid address.
        let ip_addr = match multiaddr_iter.next().expect("error extracting ip address") {
            Protocol::Ip4(ip4_addr) => IpAddr::V4(ip4_addr),
            Protocol::Ip6(ip6_addr) => IpAddr::V6(ip6_addr),
            Protocol::Dns(_) => return None,
            _ => panic!("invalid multiaddr"),
        };

        // Panic: `self` must always contain a valid address.
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

        // Panic: `self` must always contain a valid address.
        let dns = match address_iter.next().expect("error extracting ip address") {
            Protocol::Dns(dns) => dns,
            _ => return false,
        };

        // Panic: `self` must always contain a valid address.
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
            .field("address", &self.address)
            .field("public_key", &pubkey_to_base58(&self.public_key))
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
        pubkey_to_base58(public_key),
    )
}

impl FromStr for AutopeeringMultiaddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .split_terminator(&format!("/{}/", AUTOPEERING_MULTIADDR_PROTOCOL_NAME))
            .collect::<Vec<&str>>();

        if parts.len() != 2 {
            return Err(Error::AutopeeringMultiaddr);
        }

        let address = parts[0].parse().map_err(|_| Error::AutopeeringMultiaddrAddressPart)?;
        let public_key = base58_to_pubkey(parts[1])?;
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
        formatter.write_str("autopeering multiaddr")
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(de::Error::custom)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(de::Error::custom)
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(de::Error::custom)
    }
}

pub(crate) fn pubkey_to_base58(pub_key: &PublicKey) -> String {
    bs58::encode(pub_key.to_bytes()).into_string()
}

pub(crate) fn base58_to_pubkey(base58_pubkey: impl AsRef<str>) -> Result<PublicKey, Error> {
    if !PUBKEY_BASE58_SIZE_RANGE.contains(&base58_pubkey.as_ref().len()) {
        return Err(Error::Base58PublicKeyEncoding);
    }

    let mut bytes = [0u8; PUBLIC_KEY_LENGTH];

    bs58::decode(base58_pubkey.as_ref())
        .into(&mut bytes)
        .map_err(|_| Error::Base58PublicKeyEncoding)?;

    PublicKey::try_from_bytes(bytes).map_err(|_| Error::Base58PublicKeyEncoding)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Returned, if the host address part wasn't a valid multi address.
    #[error("invalid autopeering multi address address part")]
    AutopeeringMultiaddrAddressPart,
    /// Returned, if the public key part wasn't a base58 encoded ed25519 public key.
    #[error("invalid autopeering multi address public key part")]
    AutopeeringMultiaddrPubKeyPart,
    /// Returned, if it's not a valid autopeering multi address for any other reason.
    #[error("invalid autopeering multi address")]
    AutopeeringMultiaddr,
    /// Returned, if the base58 encoding of the public key was invalid.
    #[error("invalid base58 encoded public key")]
    Base58PublicKeyEncoding,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_between_base58_and_pubkey() {
        let base58_pubkey = "4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM";
        let pubkey = base58_to_pubkey(base58_pubkey).unwrap();

        assert_eq!(base58_pubkey, pubkey_to_base58(&pubkey))
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
        let _: AutopeeringMultiaddr = "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb".parse().unwrap();
        let _: AutopeeringMultiaddr = "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2".parse().unwrap();
        let _: AutopeeringMultiaddr =
            "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
                .parse()
                .unwrap();
    }
}
