// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(warnings)]

use bee_autopeering::{init, DiscoveryEvent, Local, PeeringEvent};
use bee_autopeering::{peerstore::InMemoryPeerStore, AutopeeringConfig};

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

fn setup_config() -> AutopeeringConfig {
    let config_json = r#"
    {
        "bindAddress": "0.0.0.0:14626",
        "entryNodes": [
            "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
            "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
            "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
            "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
        ],
        "entryNodesPreferIPv6": false,
        "runAsEntryNode": false
    }"#;

    serde_json::from_str(config_json).expect("error deserializing json config")
}

#[tokio::main]
async fn main() {
    setup_fern(LevelFilter::Debug);

    let local = Local::new();
    let config = setup_config();
    println!("{:?}", config);
    let version = 0;
    let network_id = 0;
    let peerstore_config = ();
    let quit_signal = Box::pin(async move { ctrl_c().await });

    let (mut discovery_rx, mut peering_rx) = bee_autopeering::init::<InMemoryPeerStore, _>(
        config,
        version,
        network_id,
        local,
        peerstore_config,
        quit_signal,
    )
    .await
    .expect("initializing autopeering system failed");

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

fn handle_discovery_event(discovery_event: DiscoveryEvent) {}
fn handle_peering_event(peering_event: PeeringEvent) {}
