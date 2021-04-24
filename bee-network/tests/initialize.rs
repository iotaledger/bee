// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "standalone")]

mod common;
use common::{await_events::*, keys_and_ids::full::*, network_config::*, shutdown::*};

use bee_network::init;

#[tokio::test]
async fn initialize() {
    let config = get_in_memory_network_config(1337);
    let config_bind_multiaddr = config.bind_multiaddr.clone();

    let keys = gen_random_keys();
    let network_id = gen_constant_net_id();

    let (_, mut rx) = init(config, keys, network_id, shutdown(10)).await;

    let bind_multiaddr = get_bind_address(&mut rx).await;
    println!("Bound to: {}", bind_multiaddr);

    let local_id = get_local_id(&mut rx).await;
    println!("Local Id: {}", local_id);

    assert_eq!(bind_multiaddr, config_bind_multiaddr);
}
