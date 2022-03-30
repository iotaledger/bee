// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    time::Duration,
};

use multiaddr::{Multiaddr, Protocol};
use regex::RegexSet;
use serde::Deserialize;

/// Default REST API binding address.
pub(crate) const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/14265";
/// Default JWT salt for REST API.
pub(crate) const DEFAULT_JWT_SALT: &str = "Bee";
/// Default routes that are available for public use and don't need JWT authentication.
pub(crate) const DEFAULT_PUBLIC_ROUTES: [&str; 11] = [
    "/health",
    "/mqtt",
    "/api/v1/info",
    "/api/v1/tips",
    "/api/v1/messages*",
    "/api/v1/transactions*",
    "/api/v1/milestones*",
    "/api/v1/outputs*",
    "/api/v1/addresses*",
    "/api/v1/treasury",
    "/api/v1/receipts*",
];
/// Default routes that are protected and need JWT authentication.
pub(crate) const DEFAULT_PROTECTED_ROUTES: [&str; 2] = ["/api/v1/*", "/api/plugins/*"];
/// Enables the proof-of-work feature on the node per default.
pub(crate) const DEFAULT_FEATURE_PROOF_OF_WORK: bool = true;
/// Default value for the white flag solidification timeout.
pub(crate) const DEFAULT_WHITE_FLAG_SOLIDIFICATION_TIMEOUT: Duration = Duration::from_secs(2);

/// REST API configuration builder.
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct RestApiConfigBuilder {
    /// REST API binding address.
    #[serde(alias = "bindAddress")]
    bind_address: Option<Multiaddr>,
    /// JWT salt for REST API.
    #[serde(alias = "jwtSalt")]
    jwt_salt: Option<String>,
    /// Routes that are available for public use and don't need JWT authentication.
    #[serde(alias = "publicRoutes")]
    public_routes: Option<Vec<String>>,
    /// Routes that are protected and need JWT authentication.
    #[serde(alias = "protectedRoutes")]
    protected_routes: Option<Vec<String>>,
    /// Enables/disables the proof-of-work feature on the node.
    #[serde(alias = "featureProofOfWork")]
    feature_proof_of_work: Option<bool>,
    /// Describes the white flag solidification timeout.
    #[serde(alias = "whiteFlagSolidificationTimeout")]
    white_flag_solidification_timeout: Option<u64>,
}

impl RestApiConfigBuilder {
    /// Creates a new config builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the binding address for the REST API.
    pub fn with_bind_address(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.bind_address.replace(addr);
            }
            Err(e) => panic!("Error parsing IP address: {:?}", e),
        }
        self
    }

    /// Sets the JWT salt.
    pub fn with_jwt_salt(mut self, jwt_salt: String) -> Self {
        self.jwt_salt.replace(jwt_salt);
        self
    }

    /// Sets all the routes that are available for public use.
    pub fn with_public_routes(mut self, routes: Vec<String>) -> Self {
        self.public_routes.replace(routes);
        self
    }

    /// Sets all the routes that need JWT authentication.
    pub fn with_protected_routes(mut self, routes: Vec<String>) -> Self {
        self.protected_routes.replace(routes);
        self
    }

    /// Set if the feature proof-of-work should be enabled or not.
    pub fn with_feature_proof_of_work(mut self, value: bool) -> Self {
        self.feature_proof_of_work.replace(value);
        self
    }

    /// Sets the while flag solidification timeout.
    pub fn with_white_flag_solidification_timeout(mut self, timeout: u64) -> Self {
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
            .expect("unsupported protocol");

        let jwt_salt = self.jwt_salt.unwrap_or_else(|| DEFAULT_JWT_SALT.to_string());

        let public_routes = {
            let routes = self
                .public_routes
                .unwrap_or_else(|| DEFAULT_PUBLIC_ROUTES.iter().map(ToString::to_string).collect());
            RegexSet::new(routes.iter().map(|r| route_to_regex(r)).collect::<Vec<_>>())
                .expect("invalid public route provided")
        };

        let protected_routes = {
            let routes = self
                .protected_routes
                .unwrap_or_else(|| DEFAULT_PROTECTED_ROUTES.iter().map(ToString::to_string).collect());
            RegexSet::new(routes.iter().map(|r| route_to_regex(r)).collect::<Vec<_>>())
                .expect("invalid protected route provided")
        };

        let feature_proof_of_work = self.feature_proof_of_work.unwrap_or(DEFAULT_FEATURE_PROOF_OF_WORK);

        let white_flag_solidification_timeout = self
            .white_flag_solidification_timeout
            .map_or_else(|| DEFAULT_WHITE_FLAG_SOLIDIFICATION_TIMEOUT, Duration::from_secs);

        RestApiConfig {
            bind_socket_addr: SocketAddr::new(address, port),
            jwt_salt,
            public_routes,
            protected_routes,
            feature_proof_of_work,
            white_flag_solidification_timeout,
        }
    }
}

/// REST API configuration.
#[derive(Clone)]
pub struct RestApiConfig {
    /// REST API binding address.
    bind_socket_addr: SocketAddr,
    /// JWT salt for REST API.
    jwt_salt: String,
    /// Routes that are available for public use and don't need JWT authentication.
    public_routes: RegexSet,
    /// Routes that are protected and need JWT authentication.
    protected_routes: RegexSet,
    /// Enables/disables the proof-of-work feature on the node.
    feature_proof_of_work: bool,
    /// Describes the white flag solidification timeout.
    white_flag_solidification_timeout: Duration,
}

impl RestApiConfig {
    /// Returns a builder for this config.
    pub fn build() -> RestApiConfigBuilder {
        RestApiConfigBuilder::new()
    }

    /// Returns the binding address.
    pub fn bind_socket_addr(&self) -> SocketAddr {
        self.bind_socket_addr
    }

    /// Returns the JWT salt.
    pub fn jwt_salt(&self) -> &str {
        &self.jwt_salt
    }

    /// Returns all the routes that are available for public use.
    pub fn public_routes(&self) -> &RegexSet {
        &self.public_routes
    }

    /// Returns all the routes that need JWT authentication.
    pub fn protected_routes(&self) -> &RegexSet {
        &self.protected_routes
    }

    /// Returns if feature "Proof-of-Work" is enabled or not.
    pub fn feature_proof_of_work(&self) -> bool {
        self.feature_proof_of_work
    }

    /// Returns the white flag solidification timeout.
    pub fn white_flag_solidification_timeout(&self) -> Duration {
        self.white_flag_solidification_timeout
    }
}

pub(crate) fn route_to_regex(route: &str) -> String {
    // Escape the string to make sure a regex can be built from it.
    // Existing wildcards `*` get escaped to `\\*`.
    let mut escaped: String = regex::escape(route);
    // Convert the escaped wildcard to a valid regex.
    escaped = escaped.replace("\\*", ".*");
    // End the regex.
    escaped.push('$');
    escaped
}
