// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "standalone")]

mod common;
use common::{await_events::*, keys_and_ids::*, network_config::*, shutdown::*};

use bee_network::{init, Command, PeerRelation};

#[tokio::test]
async fn add_peer() {
    let config1 = get_in_memory_network_config(1337);
    let keys1 = gen_random_keys();

    let config2 = get_in_memory_network_config(4242);
    let keys2 = gen_random_keys();

    let network_id = gen_constant_net_id();

    let (tx1, mut rx1) = init(config1, keys1, network_id, shutdown(10)).await;
    let (_, mut rx2) = init(config2, keys2, network_id, shutdown(10)).await;

    let peer_id1 = get_local_id(&mut rx1).await;
    let address1 = get_bind_address(&mut rx1).await;
    // println!("(1) Peer Id: {}", peer_id1);
    // println!("(1) Bound to: {}", address1);

    let peer_id2 = get_local_id(&mut rx2).await;
    let address2 = get_bind_address(&mut rx2).await;
    // println!("(2) Peer Id: {}", peer_id2);
    // println!("(2) Bound to: {}", address2);

    tx1.send(Command::AddPeer {
        alias: Some("2".into()),
        multiaddr: address2,
        relation: PeerRelation::Known,
        peer_id: peer_id2,
    });

    assert_eq!(get_added_peer_id(&mut rx1).await, peer_id2);
}
