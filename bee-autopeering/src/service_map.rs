// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{multiaddr::AddressKind, proto};

use libp2p_core::multiaddr::Protocol;

use std::{collections::HashMap, convert::TryFrom, fmt, io, net::IpAddr, str::FromStr};

/// Represents the name of a service.
pub type ServiceName = String;
type Port = u16;

pub(crate) const AUTOPEERING_SERVICE_NAME: &str = "autopeering";

/// A mapping between a service name and its bind address.
#[derive(Clone, Debug, Default)]
pub struct ServiceMap(HashMap<ServiceName, (ServiceTransport, Port)>);

impl ServiceMap {
    /// Creates a new empty service map.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Registers a service with its bind address.
    pub(crate) fn insert(&mut self, service_name: impl ToString, transport: ServiceTransport, port: Port) {
        self.0.insert(service_name.to_string(), (transport, port));
    }

    /// Returns the transport protocol of the given service.
    pub(crate) fn transport(&self, service_name: impl AsRef<str>) -> Option<ServiceTransport> {
        self.0
            .get(service_name.as_ref())
            .map(|(transport, _)| transport)
            .copied()
    }

    /// Returns the access port of a given service.
    pub(crate) fn port(&self, service_name: impl AsRef<str>) -> Option<Port> {
        self.0.get(service_name.as_ref()).map(|(_, port)| port).copied()
    }
}

impl From<proto::ServiceMap> for ServiceMap {
    fn from(services: proto::ServiceMap) -> Self {
        let proto::ServiceMap { map } = services;

        let mut services = HashMap::with_capacity(map.len());

        for (service_name, proto::NetworkAddress { network, port }) in map {
            let transport: ServiceTransport = network.parse().expect("error parsing transport protocol");
            let port = port as u16;

            services.insert(service_name, (transport, port));
        }

        Self(services)
    }
}

impl From<ServiceMap> for proto::ServiceMap {
    fn from(services: ServiceMap) -> Self {
        let ServiceMap(map) = services;

        let mut services = HashMap::with_capacity(map.len());

        for (service_name, (transport, port)) in map {
            let network_addr = proto::NetworkAddress {
                network: transport.to_string(),
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
            // TODO: include udp/tcp and port
            self.0.keys().cloned().collect::<Vec<_>>().join(";").to_string()
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ServiceTransport {
    Tcp,
    Udp,
}

impl fmt::Display for ServiceTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match self {
            ServiceTransport::Udp => "udp",
            ServiceTransport::Tcp => "tcp",
        };
        write!(f, "{}", protocol)
    }
}

impl FromStr for ServiceTransport {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tcp" => Ok(Self::Tcp),
            "udp" => Ok(Self::Udp),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unsupported transport")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto;

    impl ServiceMap {
        pub(crate) fn len(&self) -> usize {
            self.0.len()
        }
    }

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
