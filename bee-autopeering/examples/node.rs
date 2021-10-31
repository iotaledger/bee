// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(warnings)]

use bee_autopeering::{
    init, peerstore::InMemoryPeerStore, AutopeeringConfig, Event, Local, NeighborValidator, Peer, ServiceTransport,
};

use log::LevelFilter;
use serde_json::Value;
use tokio::signal::ctrl_c;
use tokio_stream::StreamExt;

use std::{future::Future, io, net::SocketAddr, pin::Pin};

fn setup_logger(level: LevelFilter) {
    fern::Dispatch::new()
        .level(level)
        .chain(io::stdout())
        .apply()
        .expect("fern");
}

fn read_config() -> AutopeeringConfig {
    // let config_json = r#"
    // {
    //     "bindAddress": "0.0.0.0:14627",
    //     "entryNodes": [
    //         "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
    //         "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
    //         "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
    //         "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
    //     ],
    //     "entryNodesPreferIPv6": false,
    //     "runAsEntryNode": false
    // }"#;

    // let config_json = r#"
    // {
    //     "bindAddress": "0.0.0.0:14627",
    //     "entryNodes": [
    //         "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb"
    //     ],
    //     "entryNodesPreferIPv6": false,
    //     "runAsEntryNode": false
    // }"#;

    let config_json = r#"
    {
        "bindAddress": "0.0.0.0:15626",
        "entryNodes": [
            "/dns/localhost/udp/14626/autopeering/CFWngmfyHryEFtgaYWmFVnHhMP5ijvuEPQuYN4UkjSyC"
        ],
        "entryNodesPreferIPv6": false,
        "runAsEntryNode": false
    }"#;

    serde_json::from_str(config_json).expect("error deserializing json config")
}

#[tokio::main]
async fn main() {
    // Set up logger.
    setup_logger(LevelFilter::Debug);

    // Read the config from a JSON file/string.
    let config = read_config();
    println!("{:#?}", config);

    // Set up a local peer, that provides the Autopeering service.
    let mut local = Local::new();
    local
        .write()
        .add_service("peering", ServiceTransport::Udp, config.bind_addr.port());

    // Network parameters.
    let version = 1;
    let network_id = "chrysalis-mainnet";

    // Storage config.
    // No config is  necessary for the `InMemoryPeerStore`.
    let peerstore_config = ();

    // Neighbor validator.
    let neighbor_validator = HippieNeighborValidator {};

    // Shutdown signal.
    let quit_signal = ctrl_c();

    // Initialize the Autopeering service.
    let mut event_rx = bee_autopeering::init::<InMemoryPeerStore, _, _, _>(
        config.clone(),
        version,
        network_id,
        local,
        peerstore_config,
        quit_signal,
        neighbor_validator,
    )
    .await
    .expect("initializing autopeering system failed");

    // Print to what IP addresses the entry nodes resolved to.
    // print_resolved_entry_nodes(config).await;

    // Enter event loop.
    'recv: loop {
        tokio::select! {
            e = event_rx.recv() => {
                if let Some(event) = e {
                    handle_event(event);
                } else {
                    break 'recv;
                }
            }
        };
    }
}

fn handle_event(event: Event) {
    match event {
        Event::PeerDiscovered { peer_id } => {
            log::info!("Peer discovered: {:?}.", peer_id);
        }
        Event::PeerDeleted { peer_id } => {
            log::info!("Peer deleted: {}.", peer_id);
        }
        Event::SaltUpdated {
            public_salt_lifetime,
            private_salt_lifetime,
        } => {
            log::info!(
                "Salts updated; new public salt lifetime: {}; new private salt lifetime: {}.",
                public_salt_lifetime,
                private_salt_lifetime
            );
        }
        Event::IncomingPeering { peer, distance } => {
            log::info!(
                "Incoming peering with peer {}; distance = {}.",
                peer.peer_id(),
                distance
            );
        }
        Event::OutgoingPeering { peer, distance } => {
            log::info!(
                "Outgoing peering with peer {}; distance = {}.",
                peer.peer_id(),
                distance
            );
        }
        Event::PeeringDropped { peer_id } => {
            log::info!("Peering dropped with {}.", peer_id);
        }
    }
}

async fn print_resolved_entry_nodes(config: AutopeeringConfig) {
    let AutopeeringConfig { entry_nodes, .. } = config;
    for mut entry_node_addr in entry_nodes {
        if entry_node_addr.resolve_dns().await {
            let resolved_addrs = entry_node_addr.resolved_addrs();
            for resolved_addr in resolved_addrs {
                println!("{} ---> {}", entry_node_addr.address(), resolved_addr);
            }
        } else {
            println!("{} ---> unresolvable", entry_node_addr.address());
        }
    }
}

struct HippieNeighborValidator {}

impl NeighborValidator for HippieNeighborValidator {
    fn is_valid(&self, _: &Peer) -> bool {
        true
    }
}
