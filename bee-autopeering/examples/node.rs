// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(warnings)]

use bee_autopeering::{
    init,
    peerstore::InMemoryPeerStore,
    peerstore::{SledPeerStore, SledPeerStoreConfig},
    AutopeeringConfig, Event, Local, NeighborValidator, Peer, ServiceTransport, AUTOPEERING_SERVICE_NAME,
};

use libp2p_core::identity::ed25519::Keypair;
use log::LevelFilter;
use serde_json::Value;
use tokio::signal::ctrl_c;
use tokio_stream::StreamExt;

use std::{future::Future, io, net::SocketAddr, pin::Pin};

const AUTOPEERING_VERSION: u32 = 1;
const NETWORK_SERVICE_NAME: &str = "chrysalis-mainnet";
const BS16_ED25519_PRIVATE_KEY: &str = "1858c941a1c4454f4df77be93481b66e5f4dcff885f30c6a6a4cb214d2fea21072e7dce6a4d4d7b89460e84bb0e3b4475b528524e7ceb741f7646400ef9f2c7b";

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
    let mut keypair = hex::decode(BS16_ED25519_PRIVATE_KEY).expect("error decoding keypair");
    let local = Local::from_keypair(Keypair::decode(&mut keypair).expect("error decoding keypair"));
    let mut write = local.write();
    write.add_service(AUTOPEERING_SERVICE_NAME, ServiceTransport::Udp, config.bind_addr.port());
    write.add_service(NETWORK_SERVICE_NAME, ServiceTransport::Tcp, 15600);
    drop(write);

    // Network parameters.
    let version = AUTOPEERING_VERSION;
    let network_name = NETWORK_SERVICE_NAME;

    // Storage config.
    // No config is  necessary for the `InMemoryPeerStore`.
    let peerstore_config = ();

    // Sled peerstore:
    // let peerstore_config = SledPeerStoreConfig::new().path("./peerstore");

    // Neighbor validator.
    let neighbor_validator = GossipNeighborValidator {};

    // Shutdown signal.
    let quit_signal = ctrl_c();

    // Initialize the Autopeering service.
    let mut event_rx = bee_autopeering::init::<InMemoryPeerStore, _, _, GossipNeighborValidator>(
        config.clone(),
        version,
        network_name,
        local,
        peerstore_config,
        quit_signal,
        Some(neighbor_validator),
    )
    .await
    .expect("initializing autopeering system failed");

    // Print to what IP addresses the entry nodes resolved to.
    print_resolved_entry_nodes(config).await;

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
    log::info!("{}", event);

    match event {
        Event::PeerDiscovered { peer_id } => {}
        Event::PeerDeleted { peer_id } => {}
        Event::SaltUpdated {
            public_salt_lifetime,
            private_salt_lifetime,
        } => {}
        Event::IncomingPeering { peer, distance } => {}
        Event::OutgoingPeering { peer, distance } => {}
        Event::PeeringDropped { peer_id } => {}
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

#[derive(Clone)]
struct GossipNeighborValidator {}

impl NeighborValidator for GossipNeighborValidator {
    fn is_valid(&self, peer: &Peer) -> bool {
        peer.has_service(NETWORK_SERVICE_NAME)
    }
}
