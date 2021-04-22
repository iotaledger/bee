// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "standalone")]

mod common;
use common::network_config;

use bee_network::{init, Command, Event, Multiaddr, NetworkConfig, NetworkEventReceiver, PeerId, PeerRelation};

#[tokio::test]
async fn send_recv() {
    // let (tx, rx) = init::init().await;
    todo!()
}
