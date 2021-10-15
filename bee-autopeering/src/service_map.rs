// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::proto;

use libp2p_core::{multiaddr::Protocol, Multiaddr};

use std::{
    collections::HashMap,
    fmt,
    net::{IpAddr, SocketAddr},
};

/// Represents the name of a service.
pub type ServiceName = String;

/// A mapping between a service name and its bind address.
#[derive(Clone, Debug, Default)]
pub struct ServiceMap(HashMap<ServiceName, Multiaddr>);

impl ServiceMap {
    /// Registers a service with its bind address.
    pub fn insert(&mut self, service_name: ServiceName, multiaddr: Multiaddr) {
        self.0.insert(service_name, multiaddr);
    }
}

impl From<proto::ServiceMap> for ServiceMap {
    fn from(services: proto::ServiceMap) -> Self {
        let proto::ServiceMap { map } = services;

        let mut services = HashMap::with_capacity(map.len());

        // From the service.proto description:
        // e.g., map[autopeering:&{tcp, 198.51.100.1:80}]
        // The service type (e.g., tcp, upd) and the address (e.g., 198.51.100.1:80)
        for (service_name, proto::NetworkAddress { network, port }) in map {
            let port = port as u16;

            let mut iter = network.split_terminator(',');

            // udp or tcp
            let transport = iter.next().expect("error unpacking transport").trim();

            // IP address
            let ip_addr: IpAddr = iter
                .next()
                .expect("error unpacking ip address")
                .trim()
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

            services.insert(service_name, multiaddr);
        }

        Self(services)
    }
}

impl From<ServiceMap> for proto::ServiceMap {
    fn from(services: ServiceMap) -> Self {
        // From the service.proto description:
        // e.g., map[autopeering:&{tcp, 198.51.100.1:80}]
        // The service type (e.g., tcp, upd) and the address (e.g., 198.51.100.1:80)

        let ServiceMap(map) = services;

        let mut services = HashMap::with_capacity(map.len());

        for (service_name, mut multiaddr) in map {
            let (port, transport) = match multiaddr.pop().expect("invalid multiaddr: port") {
                Protocol::Udp(port) => (port, "udp"),
                Protocol::Tcp(port) => (port, "tcp"),
                _ => panic!("invalid multiaddr: unsupported transport protocol"),
            };

            let addr = match multiaddr.pop().expect("invalid multiaddr: address") {
                Protocol::Ip4(ip4_addr) => ip4_addr.to_string(),
                Protocol::Ip6(ip6_addr) => ip6_addr.to_string(),
                _ => panic!("invalid multiaddr: unsupported transport protocol"),
            };

            let network_addr = proto::NetworkAddress {
                network: format!("{}, {}:{}", transport, addr, port),
                port: port as u32,
            };

            services.insert(service_name, network_addr);
        }

        Self { map: services }
    }
}

impl fmt::Display for ServiceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.keys().cloned().collect::<Vec<_>>().join(";").to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto;

    #[test]
    fn convert_service_map() {
        let mut map = HashMap::new();
        map.insert(
            "autopeering".into(),
            proto::NetworkAddress {
                network: "udp, 198.51.100.1".into(),
                port: 80,
            },
        );
        map.insert(
            "fpc".into(),
            proto::NetworkAddress {
                network: "tcp, 198.51.100.1".into(),
                port: 8000,
            },
        );
        let proto_services = proto::ServiceMap { map };

        let services: ServiceMap = proto_services.into();
        let _: proto::ServiceMap = services.into();
    }
}
