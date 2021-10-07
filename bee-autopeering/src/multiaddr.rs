use libp2p_core::multiaddr::Multiaddr;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize, Serializer,
};

use std::{borrow::Cow, fmt};

/// Go-libp2p allows Hornet to introduce a custom autopeering [`Protocol`]. In rust-libp2p we unfortunately can't do
/// that, so what we'll do is to introduce a wrapper type, then understands Hornet's custom multiaddr, and internally
/// stores it like a common `p2p` Multiaddr.
#[derive(Debug)]
pub struct AutopeeringMultiaddr(Multiaddr);

const AUTOPEERING_MULTIADDR_PROTOCOL_NAME: &str = "autopeering";

pub enum Error {
    /// Returned, if the general Multiaddr syntax is violated.
    InvalidMultiaddr,
    /// Returned, if the protocol specifier "autopeering" is missing in its `str` representation.
    InvalidAutopeeringMultiaddr,
}

impl AutopeeringMultiaddr {
    pub fn parse_from_str<'a>(s: Cow<'a, str>) -> Result<Self, Error> {
        let index = s
            .find(AUTOPEERING_MULTIADDR_PROTOCOL_NAME)
            .ok_or(Error::InvalidAutopeeringMultiaddr)?;

        let s = s.to_owned().to_string();
        let s = s.replacen(AUTOPEERING_MULTIADDR_PROTOCOL_NAME, "p2p", 1);

        if let Ok(multiaddr) = s.parse::<Multiaddr>() {
            Ok(Self(multiaddr))
        } else {
            Err(Error::InvalidMultiaddr)
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
        let s = self.0.to_string();
        let s = s.replacen("p2p", AUTOPEERING_MULTIADDR_PROTOCOL_NAME, 1);
        serializer.serialize_str(&s)
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
        let s = value.replacen(AUTOPEERING_MULTIADDR_PROTOCOL_NAME, "p2p", 1);
        Ok(AutopeeringMultiaddr(
            s.parse().expect("failed to parse autopeering multiaddr"),
        ))
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = value.replacen(AUTOPEERING_MULTIADDR_PROTOCOL_NAME, "p2p", 1);
        Ok(AutopeeringMultiaddr(
            s.parse().expect("failed to parse autopeering multiaddr"),
        ))
    }
}

// base58PubKey := "HmKTkSd9F6nnERBvVbr55FvL1hM5WfcLvsc9bc3hWxWc"
// smpl := fmt.Sprintf("/ip4/127.0.0.1/udp/14626/autopeering/%s", base58PubKey)
// ma, err := multiaddr.NewMultiaddr(smpl)

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    // NOTE: example taken from Hornet
    // ```go
    // base58PubKey := "HmKTkSd9F6nnERBvVbr55FvL1hM5WfcLvsc9bc3hWxWc"
    // smpl := fmt.Sprintf("/ip4/127.0.0.1/udp/14626/autopeering/%s", base58PubKey)
    // ma, err := multiaddr.NewMultiaddr(smpl)
    // ```
    #[test]
    fn create() {
        let base58_pubkey = "HmKTkSd9F6nnERBvVbr55FvL1hM5WfcLvsc9bc3hWxWc";
        let smpl = format!("/ip4/127.0.0.1/udp/14626/autopeering/{}", base58_pubkey);

        let ma = AutopeeringMultiaddr::parse_from_str(smpl).expect("parsing autopeering multiaddr failed");
    }
}
