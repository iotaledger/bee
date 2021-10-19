// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::mqtt::{error::Error, handlers::*, DEFAULT_MAX_INFLIGHT_REQUESTS},
    storage::StorageBackend,
};

use bee_runtime::{node::Node, worker::Worker};

use async_trait::async_trait;
use librumqttd::Broker;
use log::*;

use std::thread;

#[derive(Default)]
pub struct MqttBroker;

#[async_trait]
impl<N: Node> Worker<N> for MqttBroker
where
    N::Backend: StorageBackend,
{
    type Config = librumqttd::Config;
    type Error = Error;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let mut broker = Broker::new(config);

        let mut milestones_latest_tx = broker.link("milestones-latest").expect("linking mqtt sender failed");
        let mut milestones_confirmed_tx = broker.link("milestones-confirmed").expect("linking mqtt sender failed");
        let mut messages_tx = broker.link("messages").expect("linking mqtt sender failed");
        let mut messages_referenced_tx = broker.link("messages-referenced").expect("linking mqtt sender failed");
        let mut messages_indexation_tx = broker.link("messages-indexation").expect("linking mqtt sender failed");
        let mut messages_solidified_tx = broker.link("messages-metadata").expect("linking mqtt sender failed");
        let mut outputs_created_tx = broker.link("outputs-created").expect("linking mqtt sender failed");
        let mut outputs_consumed_tx = broker.link("outputs-consumed").expect("linking mqtt sender failed");
        let mut addresses_ouptuts_created_tx = broker
            .link("addresses-outputs-created")
            .expect("linking mqtt sender failed");
        let mut addresses_ouptuts_consumed_tx = broker
            .link("addresses-outputs-consumed")
            .expect("linking mqtt sender failed");
        let mut addresses_ed25519_ouptuts_created_tx = broker
            .link("addresses-ed25519-outputs-created")
            .expect("linking mqtt sender failed");
        let mut addresses_ed25519_ouptuts_consumed_tx = broker
            .link("addresses-ed25519-outputs-consumed")
            .expect("linking mqtt sender failed");

        thread::spawn(move || {
            debug!("Starting MQTT broker...");

            // **NOTE**: That's a blocking call until the end of the program.
            broker.start().expect("error starting broker");

            debug!("MQTT broker stopped.");
        });

        // **Note**: we are only interested in publishing, hence ignore the returned receiver.
        let _ = milestones_latest_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = milestones_confirmed_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = messages_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = messages_referenced_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = messages_indexation_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = messages_solidified_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = outputs_created_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = outputs_consumed_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = addresses_ouptuts_created_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = addresses_ouptuts_consumed_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = addresses_ed25519_ouptuts_created_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;
        let _ = addresses_ed25519_ouptuts_consumed_tx.connect(DEFAULT_MAX_INFLIGHT_REQUESTS)?;

        milestones_latest::spawn(node, milestones_latest_tx);
        milestones_confirmed::spawn(node, milestones_confirmed_tx);
        messages::spawn(node, messages_tx);
        messages_referenced::spawn(node, messages_referenced_tx);
        messages_solidified::spawn(node, messages_solidified_tx);
        messages_indexation::spawn(node, messages_indexation_tx);
        // TODO: currently it's not possible to publish the `output` on subscription since the rumqttd lib doesn't expose information about subscriptions. Relevant issue: https://github.com/bytebeamio/rumqtt/issues/304
        // outputs::spawn(...);
        outputs_created::spawn(node, outputs_created_tx);
        outputs_consumed::spawn(node, outputs_consumed_tx);
        // TODO: currently it's not possible to publish the `transactions_included_message` on subscription since the rumqttd lib doesn't expose information about subscriptions. Relevant issue: https://github.com/bytebeamio/rumqtt/issues/304
        // transactions_included_message::spawn(...);
        addresses_ouptuts_created::spawn(node, addresses_ouptuts_created_tx);
        addresses_ouptuts_consumed::spawn(node, addresses_ouptuts_consumed_tx);
        addresses_ed25519_ouptuts_created::spawn(node, addresses_ed25519_ouptuts_created_tx);
        addresses_ed25519_ouptuts_consumed::spawn(node, addresses_ed25519_ouptuts_consumed_tx);

        info!("MQTT worker started.");

        Ok(Self::default())
    }
}
