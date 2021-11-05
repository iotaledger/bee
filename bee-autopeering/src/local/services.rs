// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::proto;

use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fmt, io, str::FromStr};

/// Represents the name of a service.
pub type ServiceName = String;
pub(crate) type ServicePort = u16;

/// The announced name of the autopeering service.
pub const AUTOPEERING_SERVICE_NAME: &str = "peering";

/// A mapping between a service name and its endpoint data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServiceMap(HashMap<ServiceName, Service>);

impl ServiceMap {
    // TODO: revisit dead code
    /// Creates a new empty service map.
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Registers a service with its bind address.
    pub(crate) fn insert(&mut self, service_name: impl ToString, protocol: ServiceProtocol, port: ServicePort) {
        self.0.insert(service_name.to_string(), Service { protocol, port });
    }

    /// Returns the connection data associated with the given service.
    pub(crate) fn get(&self, service_name: impl AsRef<str>) -> Option<Service> {
        self.0.get(service_name.as_ref()).copied()
    }

    /// Returns the number of services.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<proto::ServiceMap> for ServiceMap {
    fn from(services: proto::ServiceMap) -> Self {
        let proto::ServiceMap { map } = services;

        let mut services = HashMap::with_capacity(map.len());

        for (service_name, proto::NetworkAddress { network, port }) in map {
            let protocol: ServiceProtocol = network.parse().expect("error parsing transport protocol");
            let port = port as u16;

            services.insert(service_name, Service { protocol, port });
        }

        Self(services)
    }
}

impl From<ServiceMap> for proto::ServiceMap {
    fn from(services: ServiceMap) -> Self {
        let ServiceMap(map) = services;

        let mut services = HashMap::with_capacity(map.len());

        for (service_name, Service { protocol, port }) in map {
            let network_addr = proto::NetworkAddress {
                network: protocol.to_string(),
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
            // Example: "peering/udp/14626;gossip/tcp/14625"
            self.0
                .iter()
                .map(|(service_name, service)| format!("{}/{}/{}", service_name, service.protocol, service.port))
                .collect::<Vec<_>>()
                .join(";")
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Service {
    protocol: ServiceProtocol,
    port: ServicePort,
}

impl Service {
    // TODO: revisit dead code
    #[allow(dead_code)]
    pub fn protocol(&self) -> ServiceProtocol {
        self.protocol
    }

    pub fn port(&self) -> ServicePort {
        self.port
    }
}

/// Supported protocols of announced services.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServiceProtocol {
    /// TCP
    Tcp,
    /// UDP
    Udp,
}

impl fmt::Display for ServiceProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match self {
            ServiceProtocol::Udp => "udp",
            ServiceProtocol::Tcp => "tcp",
        };
        write!(f, "{}", protocol)
    }
}

impl FromStr for ServiceProtocol {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tcp" => Ok(Self::Tcp),
            "udp" => Ok(Self::Udp),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported transport protocol",
            )),
        }
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
                network: "udp".into(),
                port: 80,
            },
        );
        map.insert(
            "fpc".into(),
            proto::NetworkAddress {
                network: "tcp".into(),
                port: 8000,
            },
        );
        let proto_services = proto::ServiceMap { map };

        let services: ServiceMap = proto_services.into();
        let _: proto::ServiceMap = services.into();
    }
}
