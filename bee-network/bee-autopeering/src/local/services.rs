// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::proto;

use libp2p_core::multiaddr::Protocol;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fmt, io, str::FromStr};

/// Represents the name of a service.
pub type ServiceName = String;
pub(crate) type ServicePort = u16;

/// The announced name of the autopeering service.
pub const AUTOPEERING_SERVICE_NAME: &str = "peering";

/// A mapping between a service name and its endpoint data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServiceMap(HashMap<ServiceName, ServiceEndpoint>);

impl ServiceMap {
    /// Registers a service with its bind address.
    pub(crate) fn insert(&mut self, service_name: impl ToString, protocol: ServiceProtocol, port: ServicePort) {
        self.0
            .insert(service_name.to_string(), ServiceEndpoint { protocol, port });
    }

    /// Returns the connection data associated with the given service name.
    pub fn get(&self, service_name: impl AsRef<str>) -> Option<ServiceEndpoint> {
        self.0.get(service_name.as_ref()).copied()
    }

    /// Returns the number of services.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl TryFrom<proto::ServiceMap> for ServiceMap {
    type Error = Error;

    fn try_from(services: proto::ServiceMap) -> Result<Self, Self::Error> {
        let proto::ServiceMap { map } = services;

        let mut services = HashMap::with_capacity(map.len());

        for (service_name, proto::NetworkAddress { network, port }) in map {
            let protocol: ServiceProtocol = network.parse().map_err(|_| Error::ServiceProtocol)?;

            if port > u16::MAX as u32 {
                return Err(Error::PortNumber);
            }
            let port = port as u16;

            services.insert(service_name, ServiceEndpoint { protocol, port });
        }

        Ok(Self(services))
    }
}

impl From<&ServiceMap> for proto::ServiceMap {
    fn from(services: &ServiceMap) -> Self {
        let ServiceMap(map) = services;

        let mut services = HashMap::with_capacity(map.len());

        for (service_name, ServiceEndpoint { protocol, port }) in map {
            let network_addr = proto::NetworkAddress {
                network: protocol.to_string(),
                port: *port as u32,
            };

            services.insert(service_name.to_owned(), network_addr);
        }

        Self { map: services }
    }
}

// Example: "peering/udp/14626;gossip/tcp/14625"
impl fmt::Display for ServiceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(service_name, service)| format!("{}/{}/{}", service_name, service.protocol, service.port))
                .reduce(|acc, service_spec| acc + ";" + &service_spec)
                .unwrap_or_default()
        )
    }
}

// TODO: consider reducing this into an enum that holds the port number.
/// Represents a service provided by a peer.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    protocol: ServiceProtocol,
    port: ServicePort,
}

impl ServiceEndpoint {
    /// The transport protocol used to access the service, e.g. TCP or UDP.
    pub fn protocol(&self) -> ServiceProtocol {
        self.protocol
    }

    /// The access port of the service.
    pub fn port(&self) -> ServicePort {
        self.port
    }

    /// Creates the corresponding `libp2p_core::multiaddr::Protocol` of this service endpoint.
    pub fn to_libp2p_protocol(&self) -> Protocol<'_> {
        match self.protocol {
            ServiceProtocol::Tcp => Protocol::Tcp(self.port),
            ServiceProtocol::Udp => Protocol::Udp(self.port),
        }
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parsing service protocol failed")]
    ServiceProtocol,
    #[error("invalid port number")]
    PortNumber,
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

        let services: &ServiceMap = &proto_services.try_into().unwrap();
        let _: proto::ServiceMap = services.into();
    }
}
