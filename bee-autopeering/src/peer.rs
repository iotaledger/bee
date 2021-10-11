// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::proto;

use bytes::BytesMut;
use crypto::signatures::ed25519::PublicKey;
use libp2p_core::{multiaddr::Protocol, Multiaddr};
use prost::{DecodeError, EncodeError, Message};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, convert::TryInto, fmt, net::IpAddr};

type ServiceName = String;

/// Represents a discovered peer.
// #[derive(Serialize, Deserialize)]
pub struct DiscoveredPeer {
    ip_address: IpAddr,
    public_key: PublicKey,
    services: HashMap<ServiceName, Multiaddr>,
}

impl DiscoveredPeer {
    /// Creates a new instance of a discovered peer.
    pub fn new(address: IpAddr, public_key: PublicKey) -> Self {
        Self {
            ip_address: address,
            public_key,
            services: HashMap::default(),
        }
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
        let mut services_mapped = HashMap::default();

        let proto::ServiceMap { map } = services.expect("missing service descriptor");

        // From the service.proto description:
        // e.g., map[autopeering:&{tcp, 198.51.100.1:80}]
        // The service type (e.g., tcp, upd) and the address (e.g., 198.51.100.1:80)
        for (service_name, proto::NetworkAddress { network, port }) in map {
            let port = port as u16;
            let mut iter = network.split_terminator(", ");
            let transport = iter.next().expect("error unpacking transport");
            let ip_addr: IpAddr = iter
                .next()
                .expect("error unpacking ip address")
                .parse()
                .expect("error parsing ip address");

            // Create libp2p's Multiaddr from the given data.
            let mut multiaddr = Multiaddr::empty();
            match ip_addr {
                IpAddr::V4(ip4_addr) => {
                    multiaddr.push(Protocol::Ip4(ip4_addr));
                }
                IpAddr::V6(ip6_addr) => {
                    multiaddr.push(Protocol::Ip6(ip6_addr));
                }
            }
            match transport {
                "udp" => multiaddr.push(Protocol::Udp(port)),
                "tcp" => multiaddr.push(Protocol::Tcp(port)),
                _ => unimplemented!("unsupported protocol"),
            }

            services_mapped.insert(service_name, multiaddr);
        }

        Ok(Self {
            ip_address,
            public_key,
            services: services_mapped,
        })
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
    pub fn services(&self) -> &HashMap<ServiceName, Multiaddr> {
        &self.services
    }

    /// Returns the Protobuf representation of this discovered peer.
    pub fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        // From the service.proto description:
        // e.g., map[autopeering:&{tcp, 198.51.100.1:80}]
        // The service type (e.g., tcp, upd) and the address (e.g., 198.51.100.1:80)

        let peer = proto::Peer {
            ip: self.ip_address.to_string(),
            public_key: self.public_key.to_bytes().to_vec(),
            // TODO
            services: None,
        };
        let mut buf = BytesMut::with_capacity(peer.encoded_len());
        peer.encode(&mut buf)?;

        Ok(buf)
    }
}

impl fmt::Debug for DiscoveredPeer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscoveredPeer")
            .field("ip_address", &self.ip_address)
            .field("public_key", &bs58::encode(&self.public_key).into_string())
            .field("services", &self.services.keys().cloned().collect::<Vec<_>>().join(";"))
            .finish()
    }
}
