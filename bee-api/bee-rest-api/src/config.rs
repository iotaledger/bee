// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub(crate) const DEFAULT_BINDING_PORT: u16 = 14265;
pub(crate) const DEFAULT_BINDING_IP_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub(crate) const DEFAULT_FEATURE_PROOF_OF_WORK: bool = true;

/// REST API configuration builder.
#[derive(Default, Deserialize)]
pub struct RestApiConfigBuilder {
    binding_port: Option<u16>,
    binding_ip_addr: Option<IpAddr>,
    feature_proof_of_work: Option<bool>,
}

impl RestApiConfigBuilder {
    /// Creates a new config builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the binding port for the REST API.
    pub fn binding_port(mut self, port: u16) -> Self {
        self.binding_port.replace(port);
        self
    }

    /// Sets the binding IP address for the REST API.
    pub fn binding_ip_addr(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.binding_ip_addr.replace(addr);
            }
            Err(e) => panic!("Error parsing IP address: {:?}", e),
        }
        self
    }

    /// Set if the feature proof-of-work should be enabled or not.
    pub fn feature_proof_of_work(mut self, value: bool) -> Self {
        self.feature_proof_of_work.replace(value);
        self
    }

    /// Builds the REST API config.
    pub fn finish(self) -> RestApiConfig {
        let binding_socket_addr = match self.binding_ip_addr.unwrap_or(DEFAULT_BINDING_IP_ADDR) {
            IpAddr::V4(ip) => SocketAddr::new(IpAddr::V4(ip), self.binding_port.unwrap_or(DEFAULT_BINDING_PORT)),
            IpAddr::V6(ip) => SocketAddr::new(IpAddr::V6(ip), self.binding_port.unwrap_or(DEFAULT_BINDING_PORT)),
        };
        let feature_proof_of_work = self.feature_proof_of_work.unwrap_or(DEFAULT_FEATURE_PROOF_OF_WORK);
        RestApiConfig {
            binding_socket_addr,
            feature_proof_of_work,
        }
    }
}

/// REST API configuration.
#[derive(Clone, Copy, Debug)]
pub struct RestApiConfig {
    pub(crate) binding_socket_addr: SocketAddr,
    pub(crate) feature_proof_of_work: bool,
}

impl RestApiConfig {
    /// Returns a builder for this config.
    pub fn build() -> RestApiConfigBuilder {
        RestApiConfigBuilder::new()
    }
    /// Returns the binding address.
    pub fn binding_socket_addr(&self) -> SocketAddr {
        self.binding_socket_addr
    }
    /// Returns if feature "Proof-of-Work" is enabled or not
    pub fn feature_proof_of_work(&self) -> bool {
        self.feature_proof_of_work
    }
}
