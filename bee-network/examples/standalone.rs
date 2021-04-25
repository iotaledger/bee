// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! To run this example open two terminals and run the following commands:
//!
//! Terminal 1:
//! ```bash
//! cargo r --example standalone --features standalone -- 1337 1338
//! ```
//!
//! Terminal 2:
//! ```bash
//! cargo r --example standalone --features standalone -- 1338 1337
//! ```
//!
//! Both

#![cfg(feature = "standalone")]
#![allow(dead_code, unused_imports)]

mod common;
use common::keys_and_ids::full::{gen_constant_net_id, gen_deterministic_keys, gen_deterministic_peer_id};

use bee_network::{init, Keypair, Multiaddr, NetworkConfig, PeerId, Protocol, PublicKey};

use log::*;
use std::{env, net::Ipv4Addr};
use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() {
    fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .expect("fern");

    let mut args = env::args().skip(1);

    let bind_port = args.next().expect("bind port missing").parse::<u16>().expect("parse");
    let bind_addr = Protocol::Ip4("127.0.0.1".parse::<Ipv4Addr>().expect("parse"));

    let peer_port = args.next().expect("peer port missing").parse::<u16>().expect("parse");
    let peer_id = gen_deterministic_peer_id(peer_port);
    let mut peer_addr = {
        let mut m = Multiaddr::empty();
        m.push(bind_addr.clone());
        m.push(Protocol::Tcp(peer_port));
        m
    };
    let peer_alias = peer_port.to_string();

    let mut config = NetworkConfig::default();
    config.replace_addr(bind_addr);
    config.replace_port(bind_port);
    config.add_peer(peer_id, peer_addr, peer_alias);

    let config_bind_multiaddr = config.bind_multiaddr.clone();

    let keys = gen_deterministic_keys(bind_port);
    let network_id = gen_constant_net_id();
    let shutdown = Box::new(Box::pin(async move {
        let _ = ctrl_c().await;
    }));

    let (tx, mut rx) = init(config, keys, network_id, shutdown).await;

    while let Some(event) = rx.recv().await {
        println!("{:?}", event);
    }
}
