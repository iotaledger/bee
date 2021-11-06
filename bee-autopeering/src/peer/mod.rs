// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod peerlist;

pub mod peer_id;
pub mod peerstore;

pub use peer_id::PeerId;
pub use peerstore::PeerStore;

use peerlist::{ActivePeersList, ReplacementList};

use crate::{
    local::{
        services::{ServiceMap, ServiceProtocol},
        Local,
    },
    proto,
};

use bytes::BytesMut;
use crypto::signatures::ed25519::PublicKey;
use prost::{DecodeError, EncodeError, Message};
use serde::{de::Visitor, ser::SerializeStruct, Deserialize, Serialize};

use std::{convert::TryInto, fmt, net::IpAddr};

/// Represents a peer.
#[derive(Clone)]
pub struct Peer {
    peer_id: PeerId,
    ip_address: IpAddr,
    services: ServiceMap,
}

impl Peer {
    /// Creates a new instance.
    pub fn new(address: IpAddr, public_key: PublicKey) -> Self {
        let peer_id = PeerId::from_public_key(public_key);

        Self {
            peer_id,
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
        self.peer_id.public_key()
    }

    /// Returns the address of this peer.
    pub fn ip_address(&self) -> IpAddr {
        self.ip_address
    }

    /// Returns the port of a service provided by this peer.
    pub fn port(&self, service_name: impl AsRef<str>) -> Option<u16> {
        self.services().get(service_name).map(|s| s.port())
    }

    /// Returns the services of this peer.
    pub fn services(&self) -> &ServiceMap {
        &self.services
    }

    /// Sets the services of this peer all at once.
    pub(crate) fn set_services(&mut self, services: ServiceMap) {
        self.services = services;
    }

    /// Returns whether the peer provides a corresponding service.
    pub fn has_service(&self, service_name: impl AsRef<str>) -> bool {
        self.services.get(service_name).is_some()
    }

    /// Adds a service with address binding to this peer.
    pub fn add_service(&mut self, service_name: impl ToString, protocol: ServiceProtocol, port: u16) {
        self.services.insert(service_name.to_string(), protocol, port);
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
            public_key: self.public_key().as_ref().to_vec(),
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
            .field("public_key", &bs58::encode(self.public_key().as_ref()).into_string())
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
            // public_key,
            ip_address,
            services,
        }
    }
}

impl From<Peer> for proto::Peer {
    fn from(peer: Peer) -> Self {
        Self {
            ip: peer.ip_address.to_string(),
            public_key: peer.public_key().as_ref().to_vec(),
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

impl From<Peer> for sled::IVec {
    fn from(peer: Peer) -> Self {
        let bytes = bincode::serialize(&peer).expect("serialization error");
        sled::IVec::from_iter(bytes.into_iter())
    }
}

impl From<sled::IVec> for Peer {
    fn from(bytes: sled::IVec) -> Self {
        bincode::deserialize(&bytes).expect("deserialization error")
    }
}

impl<'de> Deserialize<'de> for Peer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("Peer", &["peer_id", "ip_address", "services"], PeerVisitor {})
    }
}

impl Serialize for Peer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut this = serializer.serialize_struct("Peer", 3)?;
        this.serialize_field("peer_id", &self.peer_id)?;
        this.serialize_field("ip_address", &self.ip_address)?;
        this.serialize_field("services", &self.services)?;
        this.end()
    }
}

struct PeerVisitor {}

impl<'de> Visitor<'de> for PeerVisitor {
    type Value = Peer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("'Peer'")
    }
}

/// Returns whether the given peer id is known locally.
pub(crate) fn is_known(
    peer_id: &PeerId,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
) -> bool {
    // The master list doesn't need to be queried, because those always a subset of the active peers.
    peer_id == local.read().peer_id() || active_peers.read().contains(peer_id) || replacements.read().contains(peer_id)
}

// Hive.go: whether the peer has recently done an endpoint proof
// ---
/// Returns whether the corresponding peer sent a (still valid) verification response.
///
/// Also returns `false`, if the provided `peer_id` is not found in the active peer list.
pub(crate) fn is_verified(peer_id: &PeerId, active_peers: &ActivePeersList) -> bool {
    active_peers
        .read()
        .find(peer_id)
        .map_or(false, |e| e.metrics().is_verified())
}

// Hive.go: whether the given peer has recently verified the local peer
// ---
// TODO: revisit dead code
// ---
/// Returns whether the corresponding peer sent a (still valid) verification request.
///
/// Also returns `false`, if the provided `peer_id` is not found in the active peer list.
#[allow(dead_code)]
pub(crate) fn has_verified(peer_id: &PeerId, active_peers: &ActivePeersList) -> bool {
    active_peers
        .read()
        .find(peer_id)
        .map_or(false, |e| e.metrics().has_verified())
}

// Hive.go: moves the peer with the given ID to the front of the list of managed peers.
// ---
/// Performs 3 operations:
/// * Rotates the active peer list such that `peer_id` is at the front of the list (index 0);;
/// * Updates the "last_verification_response" timestamp;
/// * Increments the "verified" counter;
pub(crate) fn set_front_and_update(peer_id: &PeerId, active_peers: &ActivePeersList) -> Option<usize> {
    if let Some(p) = active_peers.write().set_newest_and_get_mut(peer_id) {
        let metrics = p.metrics_mut();
        metrics.set_last_verif_response_timestamp();
        let new_count = metrics.increment_verified_count();

        Some(new_count)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::local::services::AUTOPEERING_SERVICE_NAME;

    use super::*;
    use crypto::signatures::ed25519::SecretKey as PrivateKey;

    impl Peer {
        pub(crate) fn new_test_peer(index: u8) -> Self {
            let mut services = ServiceMap::new();
            services.insert(AUTOPEERING_SERVICE_NAME, ServiceProtocol::Udp, 1337);

            let public_key = PrivateKey::generate().unwrap().public_key();
            let peer_id = PeerId::from_public_key(public_key);

            Self {
                peer_id,
                ip_address: format!("127.0.0.{}", index).parse().unwrap(),
                services,
            }
        }

        pub(crate) fn num_services(&self) -> usize {
            self.services().len()
        }
    }
}