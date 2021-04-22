// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_network::{Multiaddr, NetworkConfig, Protocol};

pub fn get_network_config_with_port(port: u16) -> NetworkConfig {
    let mut this = NetworkConfig::default();
    this.bind_multiaddr.pop().unwrap();
    this.bind_multiaddr.push(Protocol::Tcp(port));
    this
}

pub fn get_in_memory_network_config() -> NetworkConfig {
    let mut this = NetworkConfig::default();
    this.bind_multiaddr = Multiaddr::from(Protocol::Memory(0));
    this
}
