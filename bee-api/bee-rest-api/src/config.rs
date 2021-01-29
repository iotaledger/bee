// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub(crate) const DEFAULT_BINDING_PORT: u16 = 14265;
pub(crate) const DEFAULT_BINDING_IP_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

// all available routes
pub(crate) const ROUTE_ADD_PEER: &str = "/api/v1/peer";
pub(crate) const ROUTE_BALANCE_BECH32: &str = "/api/v1/addresses/:address";
pub(crate) const ROUTE_BALANCE_ED25519: &str = "/api/v1/addresses/ed25519/:address";
pub(crate) const ROUTE_HEALTH: &str = "/health";
pub(crate) const ROUTE_INFO: &str = "/api/v1/info";
pub(crate) const ROUTE_MESSAGE: &str = "/api/v1/messages/:messageId";
pub(crate) const ROUTE_MESSAGE_CHILDREN: &str = "/api/v1/messages/:messageId/children";
pub(crate) const ROUTE_MESSAGE_METADATA: &str = "/api/v1/messages/:messageId/metadata";
pub(crate) const ROUTE_MESSAGE_RAW: &str = "/api/v1/messages/:messageId/raw";
pub(crate) const ROUTE_MESSAGES_FIND: &str = "/api/v1/messages";
pub(crate) const ROUTE_MILESTONE: &str = "/api/v1/milestones/:milestoneIndex";
pub(crate) const ROUTE_OUTPUT: &str = "/api/v1/outputs/:outputId";
pub(crate) const ROUTE_OUTPUTS_BECH32: &str = "/api/v1/addresses/:address/outputs";
pub(crate) const ROUTE_OUTPUTS_ED25519: &str = "/api/v1/addresses/ed25519/:address/outputs";
pub(crate) const ROUTE_PEER: &str = "/api/v1/peer/:peerId";
pub(crate) const ROUTE_PEERS: &str = "/api/v1/peers";
pub(crate) const ROUTE_REMOVE_PEER: &str = "/api/v1/peer/:peerId";
pub(crate) const ROUTE_SUBMIT_MESSAGE: &str = "/api/v1/messages";
pub(crate) const ROUTE_SUBMIT_MESSAGE_RAW: &str = "/api/v1/messages";
pub(crate) const ROUTE_TIPS: &str = "/api/v1/tips";

/// the routes that are available for public use
pub(crate) const DEFAULT_PUBLIC_ROUTES: [&str; 16] = [
    ROUTE_BALANCE_BECH32,
    ROUTE_BALANCE_ED25519,
    ROUTE_HEALTH,
    ROUTE_INFO,
    ROUTE_MESSAGE,
    ROUTE_MESSAGE_CHILDREN,
    ROUTE_MESSAGE_METADATA,
    ROUTE_MESSAGE_RAW,
    ROUTE_MESSAGES_FIND,
    ROUTE_MILESTONE,
    ROUTE_OUTPUT,
    ROUTE_OUTPUTS_BECH32,
    ROUTE_OUTPUTS_ED25519,
    ROUTE_SUBMIT_MESSAGE,
    ROUTE_SUBMIT_MESSAGE_RAW,
    ROUTE_TIPS,
];
pub(crate) const DEFAULT_WHITELISTED_IP_ADDRESSES: [IpAddr; 1] = [IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))];
pub(crate) const DEFAULT_FEATURE_PROOF_OF_WORK: bool = true;

/// REST API configuration builder.
#[derive(Default, Deserialize)]
pub struct RestApiConfigBuilder {
    binding_port: Option<u16>,
    binding_ip_addr: Option<IpAddr>,
    public_routes: Option<Vec<String>>,
    whitelisted_ip_addresses: Option<Vec<IpAddr>>,
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

    /// Sets all the routes that are available for public use.
    pub fn public_routes(mut self, routes: Vec<String>) -> Self {
        self.public_routes.replace(routes);
        self
    }

    /// Sets the IP addresses that are permitted to have access to all routes.
    pub fn whitelisted_ip_addresses(mut self, ip_addresses: Vec<IpAddr>) -> Self {
        self.whitelisted_ip_addresses.replace(ip_addresses);
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
        let public_routes = self
            .public_routes
            .unwrap_or(DEFAULT_PUBLIC_ROUTES.iter().map(|s| s.to_string()).collect());
        let whitelisted_ip_addresses = self
            .whitelisted_ip_addresses
            .unwrap_or(DEFAULT_WHITELISTED_IP_ADDRESSES.to_vec());
        let feature_proof_of_work = self.feature_proof_of_work.unwrap_or(DEFAULT_FEATURE_PROOF_OF_WORK);
        RestApiConfig {
            binding_socket_addr,
            public_routes,
            whitelisted_ip_addresses,
            feature_proof_of_work,
        }
    }
}

/// REST API configuration.
#[derive(Clone)]
pub struct RestApiConfig {
    pub(crate) binding_socket_addr: SocketAddr,
    pub(crate) public_routes: Vec<String>,
    pub(crate) whitelisted_ip_addresses: Vec<IpAddr>,
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
    /// Returns all the routes that are available for public use.
    pub fn public_routes(&self) -> &Vec<String> {
        &self.public_routes
    }
    /// Returns all the routes that are available for public use.
    pub fn whitelisted_ip_addresses(&self) -> &Vec<IpAddr> {
        &self.whitelisted_ip_addresses
    }
    /// Returns if feature "Proof-of-Work" is enabled or not
    pub fn feature_proof_of_work(&self) -> bool {
        self.feature_proof_of_work
    }
}
