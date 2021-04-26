// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use crate::{Multiaddr, NetworkConfig, Protocol};

pub fn get_network_config_with_port(port: u16) -> NetworkConfig {
    let mut config = NetworkConfig::default();
    config.replace_port(Protocol::Tcp(port));
    config
}

pub fn get_in_memory_network_config(port: u64) -> NetworkConfig {
    NetworkConfig::build_in_memory()
        .with_bind_multiaddr({
            let mut m = Multiaddr::empty();
            m.push(Protocol::Memory(port));
            m
        })
        .finish()
}
