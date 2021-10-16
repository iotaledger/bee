// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{identity::PeerId, proto, service_map::ServiceMap};

use bytes::BytesMut;
use crypto::signatures::ed25519::PublicKey;
use prost::{DecodeError, EncodeError, Message};
// use serde::{Deserialize, Serialize};

use std::{convert::TryInto, fmt, net::IpAddr};

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

    /// Returns the address of the discovered peer.
    pub fn ip_address(&self) -> &IpAddr {
        &self.ip_address
    }

    /// Returns the public key of the discovered peer.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the services the discovered peer.
    pub fn services(&self) -> &ServiceMap {
        &self.services
    }

    /// Creates a discovered peer from its Protobuf representation/encoding.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        let proto::Peer {
            public_key,
            ip,
            services,
        } = proto::Peer::decode(bytes)?;

        // TODO: resolve DNS addresses
        let ip_address: IpAddr = ip.parse().expect("error parsing ip address");

        let public_key = PublicKey::try_from_bytes(public_key.try_into().expect("invalid public key byte length"))
            .expect("error restoring public key from bytes");

        let services: ServiceMap = services.expect("missing service map").into();

        Ok(Self {
            ip_address,
            public_key,
            services,
        })
    }

    /// Returns the Protobuf representation of this discovered peer.
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
