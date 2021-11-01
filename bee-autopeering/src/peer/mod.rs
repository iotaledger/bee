// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod peerlist;

pub mod peer_id;
pub mod peerstore;

pub use peer_id::PeerId;
pub use peerstore::PeerStore;

use crate::{
    command::{Command, CommandTx},
    discovery::manager::{self, VERIFICATION_EXPIRATION_SECS},
    local::service_map::{ServiceMap, ServiceTransport},
    proto,
    request::RequestManager,
    server::ServerTx,
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
    peer_id: PeerId,
    public_key: PublicKey,
    ip_address: IpAddr,
    services: ServiceMap,
}

impl Peer {
    /// Creates a new instance.
    pub fn new(address: IpAddr, public_key: PublicKey) -> Self {
        let peer_id = PeerId::from_public_key(public_key);

        Self {
            peer_id,
            public_key,
            ip_address: address,
            services: ServiceMap::default(),
        }
    }

    /// Returns the [`PeerId`](crate::identity::PeerId) of this peer.
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Returns the public key of this peer.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the address of this peer.
    pub fn ip_address(&self) -> IpAddr {
        self.ip_address
    }

    /// Returns the port of a service provided by this peer.
    pub fn port(&self, service_name: impl AsRef<str>) -> Option<u16> {
        self.services().get(service_name).map(|s| s.port())
    }

    /// Returns the services this peer.
    pub fn services(&self) -> &ServiceMap {
        &self.services
    }

    /// Adds a service with address binding to this peer.
    pub fn add_service(&mut self, service_name: impl ToString, transport: ServiceTransport, port: u16) {
        self.services.insert(service_name.to_string(), transport, port);
    }

    /// Creates a peer from its Protobuf representation/encoding.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(proto::Peer::decode(bytes)?.into())
    }

    /// Returns the Protobuf representation of this peer.
    pub fn to_protobuf(&self) -> Result<BytesMut, EncodeError> {
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

    pub(crate) fn into_id(self) -> PeerId {
        self.peer_id
    }
}

impl fmt::Debug for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Peer")
            .field("peer_id", &self.peer_id.to_string())
            .field("public_key", &bs58::encode(&self.public_key).into_string())
            .field("ip_address", &self.ip_address)
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

        let peer_id = PeerId::from_public_key(public_key);

        let services: ServiceMap = services.expect("missing service map").into();

        Self {
            peer_id,
            public_key,
            ip_address,
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

impl AsRef<PeerId> for Peer {
    fn as_ref(&self) -> &PeerId {
        self.peer_id()
    }
}

// Hive.go: whether the peer has recently done an endpoint proof
pub(crate) fn is_verified<S: PeerStore>(peer_id: &PeerId, peerstore: &S) -> bool {
    peerstore.fetch_last_verification_response(peer_id).map_or(false, |ts| {
        time::since(ts).map_or(false, |since| since < VERIFICATION_EXPIRATION_SECS)
    })
}

// Hive.go: whether the given peer has recently verified the local peer
pub(crate) fn has_verified<S: PeerStore>(peer_id: &PeerId, peerstore: &S) -> bool {
    peerstore.fetch_last_verification_request(peer_id).map_or(false, |ts| {
        time::since(ts).map_or(false, |since| since < VERIFICATION_EXPIRATION_SECS)
    })
}

// Hive.go: checks whether the given peer has recently sent a Ping;
// if not, we send a Ping to trigger a verification.
pub(crate) async fn ensure_verified<S: PeerStore>(
    peer_id: &PeerId,
    peerstore: &S,
    request_mngr: &RequestManager<S>,
    server_tx: &ServerTx,
) -> bool {
    if has_verified(peer_id, peerstore) {
        true
    } else {
        manager::begin_verification_request(peer_id, request_mngr, peerstore, server_tx)
            .await
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use crate::local::service_map::AUTOPEERING_SERVICE_NAME;

    use super::*;
    use crypto::signatures::ed25519::SecretKey as PrivateKey;

    impl Peer {
        pub(crate) fn new_test_peer(index: u8) -> Self {
            let mut services = ServiceMap::new();
            services.insert(AUTOPEERING_SERVICE_NAME, ServiceTransport::Udp, 1337);

            let public_key = PrivateKey::generate().unwrap().public_key();
            let peer_id = PeerId::from_public_key(public_key.clone());

            Self {
                peer_id,
                public_key,
                ip_address: format!("127.0.0.{}", index).parse().unwrap(),
                services,
            }
        }

        pub(crate) fn num_services(&self) -> usize {
            self.services().len()
        }
    }
}
