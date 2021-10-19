// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    discovery::VERIFICATION_EXPIRATION,
    identity::PeerId,
    proto,
    service_map::{ServiceMap, ServiceProtocol},
    time::{self, Timestamp},
};

use bytes::BytesMut;
use crypto::signatures::ed25519::PublicKey;
use prost::{DecodeError, EncodeError, Message};
// use serde::{Deserialize, Serialize};

use std::{
    convert::TryInto,
    fmt,
    net::{IpAddr, Ipv4Addr},
};

/// Represents a peer.
// #[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Peer {
    ip_address: IpAddr,
    public_key: PublicKey,
    services: ServiceMap,
}

impl Peer {
    /// Creates a new instance.
    pub fn new(address: IpAddr, public_key: PublicKey) -> Self {
        Self {
            ip_address: address,
            public_key,
            services: ServiceMap::default(),
        }
    }

    /// Returns the [`PeerId`](crate::identity::PeerId) of this peer.
    pub fn peer_id(&self) -> PeerId {
        PeerId::from_public_key(self.public_key)
    }

    /// Returns the address of this peer.
    pub fn ip_address(&self) -> IpAddr {
        self.ip_address
    }

    /// Returns the public key of this peer.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the services this peer.
    pub fn services(&self) -> &ServiceMap {
        &self.services
    }

    /// Adds a service with address binding to this peer.
    pub fn add_service(&mut self, service_name: impl ToString, protocol: ServiceProtocol, port: u16) {
        let ipv = match self.ip_address {
            IpAddr::V4(ip4) => "ip4",
            IpAddr::V6(ip6) => "ip6",
        };

        // Unwrap: we control what is being parsed.
        let multiaddr = format!("/{}/{}/{}/{}", ipv, self.ip_address, protocol, port)
            .parse()
            .unwrap();

        self.services.insert(service_name.to_string(), multiaddr);
    }

    /// Creates a peer from its Protobuf representation/encoding.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(proto::Peer::decode(bytes)?.into())
    }

    /// Returns the Protobuf representation of this peer.
    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let services: proto::ServiceMap = self.services.clone().into();

        let peer = proto::Peer {
            ip: self.ip_address.to_string(),
            public_key: self.public_key.to_bytes().to_vec(),
            services: Some(services),
        };

        let mut buf = BytesMut::with_capacity(peer.encoded_len());
        peer.encode(&mut buf)?;

        Ok(buf)
    }
}

impl fmt::Debug for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Peer")
            .field("ip_address", &self.ip_address)
            .field("public_key", &bs58::encode(&self.public_key).into_string())
            .field("services", &self.services.to_string())
            .finish()
    }
}

impl From<proto::Peer> for Peer {
    fn from(peer: proto::Peer) -> Self {
        let proto::Peer {
            public_key,
            ip,
            services,
        } = peer;

        // TODO: resolve DNS addresses
        let ip_address: IpAddr = ip.parse().expect("error parsing ip address");

        let public_key = PublicKey::try_from_bytes(public_key.try_into().expect("invalid public key byte length"))
            .expect("error restoring public key from bytes");

        let services: ServiceMap = services.expect("missing service map").into();

        Self {
            ip_address,
            public_key,
            services,
        }
    }
}

impl From<Peer> for proto::Peer {
    fn from(peer: Peer) -> Self {
        Self {
            ip: peer.ip_address.to_string(),
            public_key: peer.public_key.to_bytes().to_vec(),
            services: Some(peer.services.into()),
        }
    }
}

impl AsRef<Peer> for Peer {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub(crate) fn is_verified(last_verification_response: Timestamp) -> bool {
    time::since(last_verification_response) <= VERIFICATION_EXPIRATION
}

#[cfg(test)]
mod tests {
    use crate::service_map::AUTOPEERING_SERVICE_NAME;

    use super::*;
    use crypto::signatures::ed25519::SecretKey as PrivateKey;

    impl Peer {
        pub(crate) fn new_test_peer(index: u8) -> Self {
            let mut services = ServiceMap::new();

            services.insert(
                AUTOPEERING_SERVICE_NAME.into(),
                format!("/ip4/127.0.0.{}/udp/8080", index).parse().unwrap(),
            );

            Self {
                ip_address: format!("127.0.0.{}", index).parse().unwrap(),
                public_key: PrivateKey::generate().unwrap().public_key(),
                services,
            }
        }

        pub(crate) fn num_services(&self) -> usize {
            self.services().len()
        }
    }
}
