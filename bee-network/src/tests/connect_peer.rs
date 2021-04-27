// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::common::{await_events::*, keys_and_ids::*, network_config::*, shutdown::*};

use crate::{standalone::init, Command, PeerRelation};

#[tokio::test]
#[serial_test::serial]
async fn connect_peer() {
    let config1 = get_in_memory_network_config(1337);
    let keys1 = gen_random_keys();

    let config2 = get_in_memory_network_config(4242);
    let keys2 = gen_random_keys();

    let network_id = gen_constant_net_id();

    let (tx1, mut rx1) = init(config1, keys1, network_id, shutdown(10)).await;
    let (_tx2, mut rx2) = init(config2, keys2, network_id, shutdown(10)).await;

    let peer_id1 = get_local_id(&mut rx1).await;
    let address1 = get_bind_address(&mut rx1).await;
    println!("(1) Peer Id: {}", peer_id1);
    println!("(1) Bound to: {}", address1);

    let peer_id2 = get_local_id(&mut rx2).await;
    let address2 = get_bind_address(&mut rx2).await;

    println!("(2) Peer Id: {}", peer_id2);
    println!("(2) Bound to: {}", address2);

    tx1.send(Command::AddPeer {
        alias: Some("2".into()),
        multiaddr: address2,
        relation: PeerRelation::Known,
        peer_id: peer_id2,
    })
    .expect("send command");

    assert_eq!(get_connected_peer_id(&mut rx1).await, peer_id2);
}
