// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use multiaddr::{Multiaddr, Protocol};
use serde::Deserialize;

const DEFAULT_SESSION_TIMEOUT: u64 = 86400;
const DEFAULT_USER: &str = "admin";
const DEFAULT_PASSWORD_SALT: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_PASSWORD_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/8081";

/// Builder struct for creating a [`DashboardAuthConfig`].
#[derive(Default, Deserialize, Eq, PartialEq)]
pub struct DashboardAuthConfigBuilder {
    #[serde(alias = "sessionTimeout")]
    session_timeout: Option<u64>,
    user: Option<String>,
    #[serde(alias = "passwordSalt")]
    password_salt: Option<String>,
    #[serde(alias = "passwordHash")]
    password_hash: Option<String>,
}

impl DashboardAuthConfigBuilder {
    /// Creates a new [`DashboardAuthConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`DashboardAuthConfig`], consuming the [`DashboardAuthConfigBuilder`].
    pub fn finish(self) -> DashboardAuthConfig {
        DashboardAuthConfig {
            session_timeout: self.session_timeout.unwrap_or(DEFAULT_SESSION_TIMEOUT),
            user: self.user.unwrap_or_else(|| DEFAULT_USER.to_owned()),
            password_salt: self.password_salt.unwrap_or_else(|| DEFAULT_PASSWORD_SALT.to_owned()),
            password_hash: self.password_hash.unwrap_or_else(|| DEFAULT_PASSWORD_HASH.to_owned()),
        }
    }
}

/// Dashboard authorization config.
#[derive(Clone)]
pub struct DashboardAuthConfig {
    session_timeout: u64,
    user: String,
    password_salt: String,
    password_hash: String,
}

impl DashboardAuthConfig {
    /// Returns a new [`DashboardAuthConfigBuilder`].
    pub fn build() -> DashboardAuthConfigBuilder {
        DashboardAuthConfigBuilder::new()
    }

    /// Returns the timeout of the JSON web token.
    pub fn session_timeout(&self) -> u64 {
        self.session_timeout
    }

    /// Returns the user.
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Returns the password salt.
    pub fn password_salt(&self) -> &str {
        &self.password_salt
    }

    /// Returns the password hash.
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }
}

/// Builder struct for creating a [`DashboardConfig`].
#[derive(Default, Deserialize, Eq, PartialEq)]
pub struct DashboardConfigBuilder {
    #[serde(alias = "bindAddress")]
    bind_address: Option<Multiaddr>,
    auth: Option<DashboardAuthConfigBuilder>,
}

impl DashboardConfigBuilder {
    /// Creates a new [`DashboardConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`DashboardConfig`], consuming the [`DashboardConfigBuilder`].
    pub fn finish(self) -> DashboardConfig {
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
            .expect("Unsupported protocol");

        DashboardConfig {
            bind_socket_addr: SocketAddr::new(address, port),
            auth: self.auth.unwrap_or_default().finish(),
        }
    }
}

/// Dashboard configuration options.
#[derive(Clone)]
pub struct DashboardConfig {
    bind_socket_addr: SocketAddr,
    auth: DashboardAuthConfig,
}

impl DashboardConfig {
    /// Returns a new [`DashboardConfigBuilder`].
    pub fn build() -> DashboardConfigBuilder {
        DashboardConfigBuilder::new()
    }

    /// Returns the dashboard bound address.
    pub fn bind_socket_addr(&self) -> SocketAddr {
        self.bind_socket_addr
    }

    /// Returns the dashboard authorization config.
    pub fn auth(&self) -> &DashboardAuthConfig {
        &self.auth
    }
}
