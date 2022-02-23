// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use multiaddr::{Multiaddr, Protocol};
use serde::Deserialize;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

pub(crate) const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/14265";

// all available routes
pub(crate) const ROUTE_ADD_PEER: &str = "/api/v1/peers";
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
pub(crate) const ROUTE_MILESTONE_UTXO_CHANGES: &str = "/api/v1/milestones/:milestoneIndex/utxo-changes";
pub(crate) const ROUTE_OUTPUT: &str = "/api/v1/outputs/:outputId";
pub(crate) const ROUTE_OUTPUTS_BECH32: &str = "/api/v1/addresses/:address/outputs";
pub(crate) const ROUTE_OUTPUTS_ED25519: &str = "/api/v1/addresses/ed25519/:address/outputs";
pub(crate) const ROUTE_PEER: &str = "/api/v1/peers/:peerId";
pub(crate) const ROUTE_PEERS: &str = "/api/v1/peers";
pub(crate) const ROUTE_REMOVE_PEER: &str = "/api/v1/peers/:peerId";
pub(crate) const ROUTE_SUBMIT_MESSAGE: &str = "/api/v1/messages";
pub(crate) const ROUTE_SUBMIT_MESSAGE_RAW: &str = "/api/v1/messages";
pub(crate) const ROUTE_TIPS: &str = "/api/v1/tips";
pub(crate) const ROUTE_RECEIPTS: &str = "/api/v1/receipts";
pub(crate) const ROUTE_RECEIPTS_AT: &str = "/api/v1/receipts/:milestoneIndex";
pub(crate) const ROUTE_TREASURY: &str = "/api/v1/treasury";
pub(crate) const ROUTE_TRANSACTION_INCLUDED_MESSAGE: &str = "/api/v1/transactions/:transactionId/included-message";
pub(crate) const ROUTE_WHITE_FLAG: &str = "/api/plugins/debug/whiteflag";

/// the routes that are available for public use
pub(crate) const DEFAULT_PUBLIC_ROUTES: [&str; 21] = [
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
    ROUTE_MILESTONE_UTXO_CHANGES,
    ROUTE_OUTPUT,
    ROUTE_OUTPUTS_BECH32,
    ROUTE_OUTPUTS_ED25519,
    ROUTE_SUBMIT_MESSAGE,
    ROUTE_SUBMIT_MESSAGE_RAW,
    ROUTE_TIPS,
    ROUTE_RECEIPTS,
    ROUTE_RECEIPTS_AT,
    ROUTE_TREASURY,
    ROUTE_TRANSACTION_INCLUDED_MESSAGE,
];
pub(crate) const DEFAULT_ALLOWED_IPS: [IpAddr; 2] = [
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
];
pub(crate) const DEFAULT_FEATURE_PROOF_OF_WORK: bool = true;
pub(crate) const DEFAULT_WHITE_FLAG_SOLIDIFICATION_TIMEOUT: u64 = 2;

/// REST API configuration builder.
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct RestApiConfigBuilder {
    #[serde(alias = "bindAddress")]
    bind_address: Option<Multiaddr>,
    #[serde(alias = "publicRoutes")]
    public_routes: Option<Vec<String>>,
    #[serde(alias = "allowedIps")]
    allowed_ips: Option<Vec<IpAddr>>,
    #[serde(alias = "featureProofOfWork")]
    feature_proof_of_work: Option<bool>,
    #[serde(alias = "whiteFlagSolidificationTimeout")]
    white_flag_solidification_timeout: Option<u64>,
}

impl RestApiConfigBuilder {
    /// Creates a new config builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the binding address for the REST API.
    pub fn bind_address(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.bind_address.replace(addr);
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

    /// Sets the IP addresses that are allowed to access all the routes.
    pub fn allowed_ips(mut self, allowed_ips: Vec<IpAddr>) -> Self {
        self.allowed_ips.replace(allowed_ips);
        self
    }

    /// Set if the feature proof-of-work should be enabled or not.
    pub fn feature_proof_of_work(mut self, value: bool) -> Self {
        self.feature_proof_of_work.replace(value);
        self
    }

    /// Sets the while flag solidification timeout.
    pub fn white_flag_solidification_timeout(mut self, timeout: u64) -> Self {
        self.white_flag_solidification_timeout.replace(timeout);
        self
    }

    /// Builds the REST API config.
    pub fn finish(self) -> RestApiConfig {
        let multi_addr = self
            .bind_address
            // We made sure that the default value is valid and therefore parseable.
            .unwrap_or_else(|| DEFAULT_BIND_ADDRESS.parse().unwrap());
        let address = multi_addr
            .iter()
            .find_map(|x| match x {
                Protocol::Dns(address) => Some(
                    (address.to_string(), 0)
                        .to_socket_addrs()
                        .unwrap_or_else(|error| panic!("error resolving '{}':{}", address, error))
                        .next()
                        // Unwrapping here is fine, because to_socket-addrs() didn't return an error,
                        // thus we can be sure that the iterator contains at least 1 element.
                        .unwrap()
                        .ip(),
                ),
                Protocol::Ip4(ip) => Some(IpAddr::V4(ip)),
                Protocol::Ip6(ip) => Some(IpAddr::V6(ip)),
                _ => None,
            })
            .expect("Unsupported address");

        let port = multi_addr
            .iter()
            .find_map(|x| if let Protocol::Tcp(port) = x { Some(port) } else { None })
            .unwrap_or_else(|| panic!("Unsupported protocol"));
        let public_routes: Box<[String]> = self
            .public_routes
            .unwrap_or_else(|| DEFAULT_PUBLIC_ROUTES.iter().map(|s| s.to_string()).collect())
            .into_boxed_slice();
        let allowed_ips: Box<[IpAddr]> = self
            .allowed_ips
            .unwrap_or_else(|| DEFAULT_ALLOWED_IPS.to_vec())
            .into_boxed_slice();
        let feature_proof_of_work = self.feature_proof_of_work.unwrap_or(DEFAULT_FEATURE_PROOF_OF_WORK);
        let white_flag_solidification_timeout = self
            .white_flag_solidification_timeout
            .unwrap_or(DEFAULT_WHITE_FLAG_SOLIDIFICATION_TIMEOUT);

        RestApiConfig {
            binding_socket_addr: SocketAddr::new(address, port),
            public_routes,
            allowed_ips,
            feature_proof_of_work,
            white_flag_solidification_timeout,
        }
    }
}

/// REST API configuration.
#[derive(Clone)]
pub struct RestApiConfig {
    pub(crate) binding_socket_addr: SocketAddr,
    pub(crate) public_routes: Box<[String]>,
    pub(crate) allowed_ips: Box<[IpAddr]>,
    pub(crate) feature_proof_of_work: bool,
    pub(crate) white_flag_solidification_timeout: u64,
}

impl RestApiConfig {
    /// Returns a builder for this config.
    pub fn build() -> RestApiConfigBuilder {
        RestApiConfigBuilder::new()
    }

    /// Returns the binding address.
    pub fn bind_socket_addr(&self) -> SocketAddr {
        self.binding_socket_addr
    }

    /// Returns all the routes that are available for public use.
    pub fn public_routes(&self) -> &[String] {
        &self.public_routes
    }

    /// Returns the IP addresses that are allowed to access all the routes.
    pub fn allowed_ips(&self) -> &[IpAddr] {
        &self.allowed_ips
    }

    /// Returns if feature "Proof-of-Work" is enabled or not.
    pub fn feature_proof_of_work(&self) -> bool {
        self.feature_proof_of_work
    }

    /// Returns the white flag solidification timeout.
    pub fn white_flag_solidification_timeout(&self) -> u64 {
        self.white_flag_solidification_timeout
    }
}
