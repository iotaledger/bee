// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use self::{
    broker::{MqttBroker, MqttBrokerConfig},
    config::MqttConfig,
};

use crate::storage::StorageBackend;

use bee_runtime::node::{Node, NodeBuilder};

use librumqttd as mqtt;
use librumqttd::ConsoleSettings;
use log::debug;
use rumqttlog::Config as RouterSettings;

use std::{collections::HashMap, thread};

mod broker;
pub mod config;
mod error;
mod event;
mod handlers;

// Default settings for the broker.
const DEFAULT_BROKER_ID: usize = 0;

// Default settings for the router.
const DEFAULT_ROUTER_ID: usize = 0;
const DEFAULT_ROUTER_DIR: &str = "/tmp/rumqttd";
const DEFAULT_MAX_SEGMENT_SIZE: usize = 10240;
const DEFAULT_MAX_SEGMENT_COUNT: usize = 10;
const DEFAULT_MAX_CONNECTIONS: usize = 10001;

// Default settings for the server.
const DEFAULT_NEXT_CONNECTION_DELAY: u64 = 1;
const DEFAULT_CONNECTION_TIMEOUT_MS: u16 = 5000;
const DEFAULT_MAX_CLIENT_ID_LEN: usize = 256;
const DEFAULT_THROTTLE_DELAY_MS: u64 = 0;
const DEFAULT_MAX_PAYLOAD_SIZE: usize = 5120;
const DEFAULT_MAX_INFLIGHT_COUNT: u16 = 200;
const DEFAULT_MAX_INFLIGHT_SIZE: usize = 1024;
const DEFAULT_MAX_INFLIGHT_REQUESTS: usize = 200;

pub async fn init<N: Node>(config: MqttConfig, mut node_builder: N::Builder) -> N::Builder
where
    N::Backend: StorageBackend,
{
    let MqttConfig {
        server_bind_addr,
        console_bind_addr,
    } = config;

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
        listen: server_bind_addr.clone(),
        cert: None, // Option<ServerCert>,
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
        console: ConsoleSettings {
            listen: console_bind_addr,
        },
    };

    let mut broker = mqtt::Broker::new(config);

    let mut milestones_latest_tx = broker.link("milestones/latest").expect("linking mqtt sender failed");
    let mut milestones_confirmed_tx = broker.link("milestones/confirmed").expect("linking mqtt sender failed");
    let mut messages_tx = broker.link("messages").expect("linking mqtt sender failed");
    let mut messages_referenced_tx = broker.link("messages/referenced").expect("linking mqtt sender failed");
    let mut messages_indexation_tx = broker.link("indexation/{index}").expect("linking mqtt sender failed");
    let mut messages_metadata_tx = broker
        .link("messages/{id}/metadata")
        .expect("linking mqtt sender failed");
    let mut outputs_tx = broker.link("outputs/{id}").expect("linking mqtt sender failed");
    let mut outputs_created_tx = broker.link("outputs/{id}").expect("linking mqtt sender failed");
    let mut outputs_consumed_tx = broker.link("outputs/{id}").expect("linking mqtt sender failed");
    let mut transactions_included_message_tx = broker
        .link(" transactions/{transactionId}/included-message")
        .expect("linking mqtt sender failed");
    let mut addresses_ouptuts_created_tx = broker
        .link("addresses/{address}/outputs")
        .expect("linking mqtt sender failed");
    let mut addresses_ouptuts_consumed_tx = broker
        .link("addresses/{address}/outputs")
        .expect("linking mqtt sender failed");
    let mut addresses_ed25519_ouptuts_created_tx = broker
        .link("addresses/ed25519/{address}/outputs")
        .expect("linking mqtt sender failed");
    let mut addresses_ed25519_ouptuts_consumed_tx = broker
        .link("addresses/ed25519/{address}/outputs")
        .expect("linking mqtt sender failed");

    thread::spawn(move || {
        debug!("Starting MQTT broker.");

        // **NOTE**: That's a blocking call until the end of the program.
        broker.start().expect("error starting broker");

        debug!("MQTT broker stopped.");
    });

    // **Note**: we are only interested in publishing, hence ignore the returned receiver.

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

    let _ = messages_indexation_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = messages_metadata_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let outputs_rx = outputs_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = outputs_created_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = outputs_consumed_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let transactions_included_message_rx = transactions_included_message_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = addresses_ouptuts_created_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = addresses_ouptuts_consumed_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = addresses_ed25519_ouptuts_created_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let _ = addresses_ed25519_ouptuts_consumed_tx
        .connect(DEFAULT_MAX_INFLIGHT_REQUESTS)
        .expect("mqtt connect error");

    let broker_config = MqttBrokerConfig {
        milestones_latest_tx,
        milestones_confirmed_tx,
        messages_tx,
        messages_referenced_tx,
        messages_indexation_tx,
        messages_metadata_tx,
        outputs_tx,
        outputs_rx,
        outputs_created_tx,
        outputs_consumed_tx,
        transactions_included_message_tx,
        transactions_included_message_rx,
        addresses_ouptuts_created_tx,
        addresses_ouptuts_consumed_tx,
        addresses_ed25519_ouptuts_created_tx,
        addresses_ed25519_ouptuts_consumed_tx,
    };

    node_builder = node_builder.with_worker_cfg::<MqttBroker>(broker_config);
    node_builder
}
