// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{multiaddr::Protocol, Multiaddr};
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

const DEFAULT_SESSION_TIMEOUT: u64 = 86400;
const DEFAULT_USER: &str = "admin";
const DEFAULT_PASSWORD_SALT: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_PASSWORD_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_BIND_ADDRESSES: &str = "/ip4/1.0.0.0/tcp/8081";

#[derive(Default, Deserialize)]
pub struct DashboardAuthConfigBuilder {
    session_timeout: Option<u64>,
    user: Option<String>,
    password_salt: Option<String>,
    password_hash: Option<String>,
}

impl DashboardAuthConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> DashboardAuthConfig {
        DashboardAuthConfig {
            session_timeout: self.session_timeout.unwrap_or(DEFAULT_SESSION_TIMEOUT),
            user: self.user.unwrap_or_else(|| DEFAULT_USER.to_owned()),
            password_salt: self.password_salt.unwrap_or_else(|| DEFAULT_PASSWORD_SALT.to_owned()),
            password_hash: self.password_hash.unwrap_or_else(|| DEFAULT_PASSWORD_HASH.to_owned()),
        }
    }
}

#[derive(Clone)]
pub struct DashboardAuthConfig {
    session_timeout: u64,
    user: String,
    password_salt: String,
    password_hash: String,
}

impl DashboardAuthConfig {
    pub fn build() -> DashboardAuthConfigBuilder {
        DashboardAuthConfigBuilder::new()
    }

    pub fn session_timeout(&self) -> u64 {
        self.session_timeout
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn password_salt(&self) -> &str {
        &self.password_salt
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }
}

#[derive(Default, Deserialize)]
pub struct DashboardConfigBuilder {
    bind_addresses: Option<Multiaddr>,
    auth: Option<DashboardAuthConfigBuilder>,
}

impl DashboardConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> DashboardConfig {
        let multi_addr = self
            .bind_addresses
            .unwrap_or_else(|| DEFAULT_BIND_ADDRESSES.parse().unwrap());
        let address = multi_addr
            .iter()
            .find_map(|x| match x {
                Protocol::Dns(std::borrow::Cow::Borrowed("localhost")) => Some(
                    ("localhost", 0)
                        .to_socket_addrs()
                        .expect("error resolving 'localhost'")
                        .nth(0)
                        .unwrap()
                        .ip(),
                ),
                Protocol::Ip4(ip) => Some(IpAddr::V4(ip)),
                Protocol::Ip6(ip) => Some(IpAddr::V6(ip)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("Unsupported address"));

        let port = multi_addr
            .iter()
            .find_map(|x| match x {
                Protocol::Tcp(port) => Some(port),
                _ => None,
            })
            .unwrap_or_else(|| panic!("Unsupported protocol"));

        DashboardConfig {
            bind_socket_addr: SocketAddr::new(address, port),
            auth: self.auth.unwrap_or_default().finish(),
        }
    }
}

#[derive(Clone)]
pub struct DashboardConfig {
    bind_socket_addr: SocketAddr,
    auth: DashboardAuthConfig,
}

impl DashboardConfig {
    pub fn build() -> DashboardConfigBuilder {
        DashboardConfigBuilder::new()
    }

    pub fn bind_socket_addr(&self) -> SocketAddr {
        self.bind_socket_addr
    }

    pub fn auth(&self) -> &DashboardAuthConfig {
        &self.auth
    }
}
