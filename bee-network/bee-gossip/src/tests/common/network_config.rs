// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use crate::{GossipLayerConfig, Multiaddr, Protocol};

pub fn get_network_config_with_port(port: u16) -> GossipLayerConfig {
    let mut config = GossipLayerConfig::default();
    config.replace_port(Protocol::Tcp(port)).unwrap();
    config
}

pub fn get_in_memory_network_config(port: u64) -> GossipLayerConfig {
    GossipLayerConfig::build_in_memory()
        .with_bind_multiaddr({
            let mut m = Multiaddr::empty();
            m.push(Protocol::Memory(port));
            m
        })
        .finish()
}
