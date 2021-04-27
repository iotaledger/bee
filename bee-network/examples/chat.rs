// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! To run this example open two terminals and run the following commands:
//!
//! Terminal 1:
//! ```bash
//! cargo r --example chat -- 1337 4242
//! ```
//!
//! Terminal 2:
//! ```bash
//! cargo r --example chat -- 4242 1337
//! ```
//!
//! Both network instances will automatically connect to each other, and you can then
//! start sending messages between the two.
//!
//! NOTE: Due to some limitations of `tokio::io::stdin` the shudown is not working
//! currently. Use your favorite kill command, for example `killall chat`.

mod common;

#[tokio::main]
async fn main() {
    use bee_network::{alias, standalone::init, Event, Multiaddr, NetworkConfig, Protocol};
    use common::keys_and_ids::{gen_constant_net_id, gen_deterministic_keys, gen_deterministic_peer_id};
    use std::{
        env,
        io::{stdin, stdout, Write},
        net::Ipv4Addr,
        thread,
    };
    use tokio::signal::ctrl_c;
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
    let peer_addr = {
        let mut m = Multiaddr::empty();
        m.push(bind_addr.clone());
        m.push(Protocol::Tcp(peer_port));
        m
    };

    let mut config = NetworkConfig::default();
    config.replace_addr(bind_addr).expect("invalid bind address");
    config.replace_port(Protocol::Tcp(bind_port)).expect("invalid port");
    config
        .add_static_peer(peer_id, peer_addr, None)
        .expect("invalid static peer");

    let _config_bind_multiaddr = config.bind_multiaddr().clone();

    let keys = gen_deterministic_keys(bind_port);
    let network_id = gen_constant_net_id();
    let shutdown = Box::new(Box::pin(async move {
        let _ = ctrl_c().await;
    }));

    let mut _my_local_id = None;
    let (_tx, mut rx) = init(config, keys, network_id, shutdown).await;

    loop {
        if let Some(event) = rx.recv().await {
            println!("------> {:?}", event);
            match event {
                Event::LocalIdCreated { local_id } => {
                    _my_local_id = Some(local_id);
                }
                Event::PeerConnected {
                    peer_id,
                    mut gossip_in,
                    gossip_out,
                    ..
                } => {
                    println!("{} joined the chat.\n", alias!(peer_id));

                    thread::spawn(move || {
                        loop {
                            print!("Me    : ");
                            stdout().flush().unwrap();

                            let mut msg = String::new();

                            stdin().read_line(&mut msg).unwrap();
                            let msg = msg.trim_end().to_string();

                            gossip_out.send(msg.into_bytes()).expect("send message");
                        }
                    });

                    loop {
                        if let Some(msg) = (&mut gossip_in).next().await {
                            println!("\r{}: {}", alias!(peer_id), String::from_utf8(msg).unwrap());
                            print!("Me    : ");
                            stdout().flush().unwrap();
                        }
                    }
                }
                Event::PeerDisconnected { .. } => {
                    println!("{} left the chat.", alias!(peer_id));
                }
                _ => {}
            }
        }
    }
}
