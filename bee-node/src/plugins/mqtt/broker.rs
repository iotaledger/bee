// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::event::*;

use crate::storage::StorageBackend;

use bee_protocol::workers::event::{MessageProcessed, NewIndexationMessage};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::event::{ConfirmedMilestoneChanged, LatestMilestoneChanged};
use bee_common::packable::Packable;

use async_trait::async_trait;
use librumqttd as mqtt;
use log::*;
use mqtt::LinkTx;
// use rumqttlog::Data as Message;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use std::{
    any::{Any},
    convert::Infallible,
};

use bee_rest_api::types::responses::OutputResponse;
use bee_ledger::workers::event::{OutputCreated, OutputConsumed, MessageReferenced};
use chrono::format::format;
use bee_message::output::Output;
use bee_message::address::Address;
use crate::config::NodeConfig;

use crate::plugins::mqtt::handlers::*;
use librumqttd::LinkRx;

pub(crate) const ___TOPIC_MESSAGES_REFERENCED: &str = "messages/referenced";
pub(crate) const ___TOPIC_MESSAGES_METADATA: &str = "messages/{messageId}/metadata";

pub(crate) const TOPIC_OUTPUTS: &str = "outputs/{outputId}";
pub(crate) const _TOPIC_INCLUDED_MESSAGE: &str = "transactions/[transactionId]/included-message";

pub struct MqttBrokerConfig {
    pub milestones_latest_tx: LinkTx,
    pub milestones_confirmed_tx: LinkTx,
    pub messages_tx: LinkTx,
    pub messages_referenced_tx: LinkTx,
    pub messages_indexation_tx: LinkTx,
    pub messages_metadata_tx: LinkTx,
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
            addresses_ed25519_ouptuts_consumed_tx
        } = config;

        milestones_latest::spawn(node, milestones_latest_tx);
        milestones_confirmed::spawn(node, milestones_confirmed_tx);
        messages::spawn(node, messages_tx);
        messages_referenced::spawn(node, messages_referenced_tx);
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