// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use multiaddr::{Multiaddr, Protocol};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

pub(crate) const DEFAULT_SERVER_BIND_ADDR: &str = "/ip4/0.0.0.0/tcp/1883";
pub(crate) const DEFAULT_CONSOLE_BIND_ADDR: &str = "/ip4/0.0.0.0/tcp/1884";

#[derive(Default, Deserialize)]
pub struct MqttConfigBuilder {
    server_bind_addr: Option<Multiaddr>,
    console_bind_addr: Option<Multiaddr>,
}

impl MqttConfigBuilder {
    /// Creates a new config builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the binding address for the MQTT server.
    pub fn server_bind_addr(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.server_bind_addr.replace(addr);
            }
            Err(e) => panic!("Error parsing bind address: {:?}", e),
        }
        self
    }

    /// Sets the binding address for the MQTT console.
    pub fn console_bind_addr(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.console_bind_addr.replace(addr);
            }
            Err(e) => panic!("Error parsing bind address: {:?}", e),
        }
        self
    }

    pub fn finish(self) -> MqttConfig {
        // Get server address and port.
        let (server_addr, server_port) = {
            let multi_addr = self
                .server_bind_addr
                // We made sure that the default value is valid and therefore parseable.
                .unwrap_or_else(|| DEFAULT_SERVER_BIND_ADDR.parse().unwrap());
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

            (address, port)
        };

        // Get console address and port.
        let (console_addr, console_port) = {
            let multi_addr = self
                .console_bind_addr
                // We made sure that the default value is valid and therefore parseable.
                .unwrap_or_else(|| DEFAULT_CONSOLE_BIND_ADDR.parse().unwrap());
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

            (address, port)
        };

        MqttConfig {
            server_bind_addr: SocketAddr::new(server_addr, server_port),
            console_bind_addr: SocketAddr::new(console_addr, console_port),
        }
    }
}

#[derive(Clone)]
pub struct MqttConfig {
    pub(crate) server_bind_addr: SocketAddr,
    pub(crate) console_bind_addr: SocketAddr,
}
