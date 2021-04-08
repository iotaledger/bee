// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod broker;
pub mod config;
mod error;
mod event;

use bee_runtime::node::{Node, NodeBuilder};

use self::{
    broker::{MqttBroker, MqttBrokerConfig},
    config::MqttConfig,
};

use librumqttd as mqtt;
use rumqttlog::Config as RouterSettings;

use std::{collections::HashMap, thread};

const DEFAULT_NEXT_CONNECTION_DELAY: u64 = 1;
const DEFAULT_CONNECTION_TIMEOUT_MS: u16 = 100;
const DEFAULT_MAX_CLIENT_ID_LEN: usize = 256;
const DEFAULT_THROTTLE_DELAY_MS: u64 = 0;
const DEFAULT_MAX_PAYLOAD_SIZE: usize = 2048;
const DEFAULT_MAX_INFLIGHT_COUNT: u16 = 500;
const DEFAULT_MAX_INFLIGHT_SIZE: usize = 1024;
const DEFAULT_BROKER_ID: usize = 0;
const DEFAULT_ROUTER_ID: usize = 0;
const DEFAULT_ROUTER_DIR: &str = "/tmp/rumqttd";
const DEFAULT_MAX_SEGMENT_SIZE: usize = 1024 * 1024;
const DEFAULT_MAX_SEGMENT_COUNT: usize = 1024;
const DEFAULT_MAX_CONNECTIONS: usize = 50;
const DEFAULT_MAX_INFLIGHT_REQUESTS: usize = 200;

pub async fn init<N: Node>(config: MqttConfig, mut node_builder: N::Builder) -> N::Builder {
    let MqttConfig { port } = config;

    let connection_settings = mqtt::ConnectionSettings {
        connection_timeout_ms: DEFAULT_CONNECTION_TIMEOUT_MS,
        max_client_id_len: DEFAULT_MAX_CLIENT_ID_LEN,
        throttle_delay_ms: DEFAULT_THROTTLE_DELAY_MS,
        max_payload_size: DEFAULT_MAX_PAYLOAD_SIZE,
        max_inflight_count: DEFAULT_MAX_INFLIGHT_COUNT,
        max_inflight_size: DEFAULT_MAX_INFLIGHT_SIZE,
        username: None, // Option<String>,
        password: None, // Option<String>,
    };

    let server_settings = mqtt::ServerSettings {
        port,
        ca_path: None,   // Option<String>,
        cert_path: None, // Option<String>,
        key_path: None,  // Option<String>,
        next_connection_delay_ms: DEFAULT_NEXT_CONNECTION_DELAY,
        connections: connection_settings, // ConnectionSettings,
    };

    let router_settings = RouterSettings {
        id: DEFAULT_ROUTER_ID,
        dir: DEFAULT_ROUTER_DIR.into(),
        max_segment_size: DEFAULT_MAX_SEGMENT_SIZE,
        max_segment_count: DEFAULT_MAX_SEGMENT_COUNT,
        max_connections: DEFAULT_MAX_CONNECTIONS,
    };

    // TODO: TLS server
    let mut servers = HashMap::with_capacity(1);
    servers.insert("non_tls".into(), server_settings);

    let config = mqtt::Config {
        id: DEFAULT_BROKER_ID,
        router: router_settings,
        servers,
        cluster: None,    // Option<HashMap<String, MeshSettings>>,
        replicator: None, // Option<ConnectionSettings>,
        console: None,    // Option<ConsoleSettings>,
    };

    let mut broker = mqtt::Broker::new(config);

    let mut milestones_latest_tx = broker.link("milestones/latest").expect("linking mqtt sender failed");
    let mut milestones_confirmed_tx = broker.link("milestones/confirmed").expect("linking mqtt sender failed");
    let mut messages_tx = broker.link("messages").expect("linking mqtt sender failed");
    let mut messages_referenced_tx = broker.link("messages/referenced").expect("linking mqtt sender failed");

    thread::spawn(move || {
        broker.start().expect("error starting broker");
    });

    // **Note**: we are only interested in puplishing, hence ignore the returned receiver.

    let _ = milestones_latest_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = milestones_confirmed_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = messages_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = messages_referenced_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let broker_config = MqttBrokerConfig {
        milestones_latest_tx,
        milestones_confirmed_tx,
        messages_tx,
        messages_referenced_tx,
    };

    node_builder = node_builder.with_worker_cfg::<MqttBroker>(broker_config);

    node_builder
}
