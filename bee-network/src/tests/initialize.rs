// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::common::{await_events::*, keys_and_ids::*, network_config::*, shutdown::*};

use crate::standalone::init;

#[tokio::test]
#[serial_test::serial]
async fn initialize() {
    let config = get_in_memory_network_config(1337);
    let config_bind_multiaddr = config.bind_multiaddr().clone();

    let keys = get_constant_keys();
    let network_id = gen_constant_net_id();

    let (_, mut rx) = init(config, keys, network_id, shutdown(10)).await.expect("init failed");

    let local_id = get_local_id(&mut rx).await;
    // println!("Local Id: {}", local_id);

    let bind_multiaddr = get_bind_address(&mut rx).await;
    // println!("Bound to: {}", bind_multiaddr);

    assert_eq!(
        local_id,
        "12D3KooWNXQuYwdb9yjefEHf1cLKPihaUsLZ2biApnJcFrSEY5pc"
            .parse()
            .expect("parse")
    );
    assert_eq!(bind_multiaddr, config_bind_multiaddr);
}
