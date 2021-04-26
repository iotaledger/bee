// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! To run this example open two terminals and run the following commands:
//!
//! Terminal 1:
//! ```bash
//! cargo r --example standalone --features standalone -- 1337 4242
//! ```
//!
//! Terminal 2:
//! ```bash
//! cargo r --example standalone --features standalone -- 4242 1337
//! ```
//!
//! Both network instances will connect to each other, and start sending messages.

mod common;

#[cfg(feature = "standalone")]
#[tokio::main]
async fn main() {
    use bee_network::{alias, init, Command, Event, Keypair, Multiaddr, NetworkConfig, PeerId, Protocol, PublicKey};
    use common::keys_and_ids::{gen_constant_net_id, gen_deterministic_keys, gen_deterministic_peer_id};
    use std::{env, net::Ipv4Addr};
    use tokio::{
        signal::ctrl_c,
        time::{sleep, Duration},
    };
    use tokio_stream::StreamExt;

    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
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
    config.replace_port(Protocol::Tcp(bind_port));
    config.add_static_peer(peer_id, peer_addr, peer_alias);

    let config_bind_multiaddr = config.bind_multiaddr().clone();

    let keys = gen_deterministic_keys(bind_port);
    let network_id = gen_constant_net_id();
    let shutdown = Box::new(Box::pin(async move {
        let _ = ctrl_c().await;
    }));

    let mut my_local_id = None;
    let (tx, mut rx) = init(config, keys, network_id, shutdown).await;

    while let Some(event) = rx.recv().await {
        println!("------> {:?}", event);
        match event {
            Event::LocalIdCreated { local_id } => {
                my_local_id = Some(local_id);
            }
            Event::PeerConnected {
                peer_id,
                mut gossip_in,
                gossip_out,
                ..
            } => {
                println!("{} joined the chat.\n", alias!(peer_id));

                // tokio::spawn(async move {
                //     loop {
                //         let msg = format!("Hello {}!", alias!(peer_id));
                //         gossip_out.send(msg.clone().into_bytes());
                //         println!("{}: {}", alias!(my_local_id.unwrap()), msg);
                //         sleep(Duration::from_secs(5)).await;
                //     }
                // });

                // loop {
                //     if let Some(msg) = (&mut gossip_in).next().await {
                //         println!("{}: {}\n", alias!(peer_id), String::from_utf8(msg).unwrap());
                //     }
                // }
            }
            Event::PeerDisconnected { .. } => {
                println!("{} left the chat.", alias!(peer_id));
            }
            _ => {}
        }
    }
}

// Examples **must** contain a main function in order to always compile for any compilation flag.
#[cfg(not(feature = "standalone"))]
fn main() {}
