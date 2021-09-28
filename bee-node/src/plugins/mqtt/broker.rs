// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::handlers::*, storage::StorageBackend};

use bee_runtime::{node::Node, worker::Worker};

use async_trait::async_trait;
use librumqttd::{LinkRx, LinkTx};
use log::*;

use std::convert::Infallible;

pub struct MqttBrokerConfig {
    pub milestones_latest_tx: LinkTx,
    pub milestones_confirmed_tx: LinkTx,
    pub messages_tx: LinkTx,
    pub messages_referenced_tx: LinkTx,
    pub messages_indexation_tx: LinkTx,
    pub messages_solidified_tx: LinkTx,
    pub outputs_tx: LinkTx,
    pub outputs_rx: LinkRx,
    pub outputs_created_tx: LinkTx,
    pub outputs_consumed_tx: LinkTx,
    pub transactions_included_message_tx: LinkTx,
    pub transactions_included_message_rx: LinkRx,
    pub addresses_ouptuts_created_tx: LinkTx,
    pub addresses_ouptuts_consumed_tx: LinkTx,
    pub addresses_ed25519_ouptuts_created_tx: LinkTx,
    pub addresses_ed25519_ouptuts_consumed_tx: LinkTx,
}

#[derive(Default)]
pub struct MqttBroker;

#[async_trait]
impl<N: Node> Worker<N> for MqttBroker
where
    N::Backend: StorageBackend,
{
    type Config = MqttBrokerConfig;
    type Error = Infallible;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let MqttBrokerConfig {
            milestones_latest_tx,
            milestones_confirmed_tx,
            messages_tx,
            messages_referenced_tx,
            messages_indexation_tx,
            messages_solidified_tx,
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
        } = config;

        milestones_latest::spawn(node, milestones_latest_tx);
        milestones_confirmed::spawn(node, milestones_confirmed_tx);
        messages::spawn(node, messages_tx);
        messages_referenced::spawn(node, messages_referenced_tx);
        messages_solidified::spawn(node, messages_solidified_tx);
        messages_indexation::spawn(node, messages_indexation_tx);
        outputs::spawn(node, outputs_tx, outputs_rx);
        outputs_created::spawn(node, outputs_created_tx);
        outputs_consumed::spawn(node, outputs_consumed_tx);
        transactions_included_message::spawn(node, transactions_included_message_tx, transactions_included_message_rx);
        addresses_ouptuts_created::spawn(node, addresses_ouptuts_created_tx);
        addresses_ouptuts_consumed::spawn(node, addresses_ouptuts_consumed_tx);
        addresses_ed25519_ouptuts_created::spawn(node, addresses_ed25519_ouptuts_created_tx);
        addresses_ed25519_ouptuts_consumed::spawn(node, addresses_ed25519_ouptuts_consumed_tx);

        info!("MQTT worker started.");

        Ok(Self::default())
    }
}
