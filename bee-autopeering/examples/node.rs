// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(warnings)]

use bee_autopeering::{
    init, peerstore::InMemoryPeerStore, AutopeeringConfig, DiscoveryEvent, Local, PeeringEvent, ServiceTransport,
};

use log::LevelFilter;
use serde_json::Value;
use tokio::signal::ctrl_c;
use tokio_stream::StreamExt;

use std::{future::Future, io, net::SocketAddr, pin::Pin};

fn setup_fern(level: LevelFilter) {
    fern::Dispatch::new()
        .level(level)
        .chain(io::stdout())
        .apply()
        .expect("fern");
}

fn read_config() -> AutopeeringConfig {
    // let config_json = r#"
    // {
    // "bindAddress": "0.0.0.0:14627",
    // "entryNodes": [
    // "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
    // "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/
    // iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb", "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/
    // 14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2", "/dns/entry-mainnet.tanglebay.com/udp/14626/
    // autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"     ], "entryNodesPreferIPv6": false,
    // "runAsEntryNode": false
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
    setup_fern(LevelFilter::Debug);

    // Read the config from a JSON file/string.
    let config = read_config();
    println!("{:#?}", config);

    // Set up a local peer, that provides the Autopeering service.
    let mut local = Local::new();
    local.add_service("peering", ServiceTransport::Udp, config.bind_addr.port());

    // Network parameters.
    let version = 1;
    let network_id = "chrysalis-mainnet";

    // Storage config.
    // No config is  necessary for the `InMemoryPeerStore`.
    let peerstore_config = ();

    // Shutdown signal.
    let quit_signal = ctrl_c();

    // Initialize the Autopeering service.
    let (mut discovery_rx, mut peering_rx) = bee_autopeering::init::<InMemoryPeerStore, _, _>(
        config.clone(),
        version,
        network_id,
        local,
        peerstore_config,
        quit_signal,
    )
    .await
    .expect("initializing autopeering system failed");

    // Print to what IP addresses the entry nodes resolved to.
    print_resolved_entry_nodes(config).await;

    // Enter event loop.
    loop {
        tokio::select! {
            de = discovery_rx.recv() => {
                if let Some(discovery_event) = de {
                    handle_discovery_event(discovery_event);
                } else {
                    break;
                }
            }
            pe = peering_rx.recv() => {
                if let Some(peering_event) = pe {
                    handle_peering_event(peering_event);
                } else {
                    break;
                }
            }
        };
    }
}

fn handle_discovery_event(discovery_event: DiscoveryEvent) {
    match discovery_event {
        DiscoveryEvent::PeerDiscovered { peer } => {
            log::info!("Peer discovered: {:?}.", peer);
        }
        DiscoveryEvent::PeerDeleted { peer_id } => {
            log::info!("Peer deleted: {}.", peer_id);
        }
    }
}

fn handle_peering_event(peering_event: PeeringEvent) {
    match peering_event {
        PeeringEvent::SaltUpdated => {
            log::info!("Salt updated.");
        }
        PeeringEvent::IncomingPeering => {
            log::info!("Incoming peering.");
        }
        PeeringEvent::OutgoingPeering => {
            log::info!("Outgoing peering.");
        }
        PeeringEvent::Dropped => {
            log::info!("Peering dropped.");
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
