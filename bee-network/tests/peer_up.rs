// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// #![cfg(feature = "standalone")]
// #![cfg(not(feature = "integrated"))]

use bee_network::{__init, util, Command, Event, Multiaddr, NetworkConfig, NetworkEventReceiver, PeerId, PeerRelation};

#[tokio::test]
async fn peer_up() {
    // TODO: use memory-address
    let config1 = NetworkConfig::new_with_port(1337);
    let keys1 = util::gen_random_keys();

    let config2 = NetworkConfig::new_with_port(1338);
    let keys2 = util::gen_random_keys();

    let network_id = util::gen_random_net_id();

    let (tx1, rx1) = __init(config1, keys1, network_id).await;
    let (tx2, rx2) = __init(config2, keys2, network_id).await;

    let _ = get_bind_address(&mut rx1).await;

    let address2 = get_bind_address(&mut rx2).await;
    let peer_id2 = get_peer_id(&mut rx2).await;

    tx1.send(Command::AddPeer {
        alias: Some("2".into()),
        multiaddr: address2,
        relation: PeerRelation::Known,
        peer_id: peer_id2,
    });

    assert_eq!(get_connected_peer_id(&mut rx1).await, peer_id2);
}

// TODO: timeout
async fn get_bind_address(rx: &mut NetworkEventReceiver) -> Multiaddr {
    loop {
        if let Some(Event::AddressBound { address }) = rx.recv().await {
            return address;
        }
    }
}

// TODO: timeout
async fn get_peer_id(rx: &mut NetworkEventReceiver) -> PeerId {
    loop {
        if let Some(Event::PeerCreated { peer_id }) = rx.recv().await {
            return peer_id;
        }
    }
}

// TODO: timeout
async fn get_connected_peer_id(rx: &mut NetworkEventReceiver) -> PeerId {
    loop {
        if let Some(Event::PeerConnected { peer_id, .. }) = rx.recv().await {
            return peer_id;
        }
    }
}
