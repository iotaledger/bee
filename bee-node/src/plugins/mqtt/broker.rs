// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::event::*;

use bee_protocol::workers::event::{IndexationMessage, MessageConfirmed, MessageProcessed};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::event::{LatestMilestoneChanged, LatestSolidMilestoneChanged};

use async_trait::async_trait;
use librumqttd as mqtt;
use log::*;
use mqtt::LinkTx;
// use rumqttlog::Data as Message;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use std::{
    any::{Any, TypeId},
    convert::Infallible,
};

pub(crate) const TOPIC_MILESTONES_LATEST: &str = "milestones/latest";
pub(crate) const TOPIC_MILESTONES_CONFIRMED: &str = "milestones/confirmed";
pub(crate) const TOPIC_MESSAGES: &str = "messages";
pub(crate) const TOPIC_MESSAGES_REFERENCED: &str = "messages/referenced";
pub(crate) const TOPIC_MESSAGES_INDEXATION: &str = "messages/indexation/{index}";
pub(crate) const TOPIC_MESSAGES_METADATA: &str = "messages/{messageId}/metadata";
pub(crate) const TOPIC_OUTPUTS: &str = "outputs/{outputId}";
pub(crate) const TOPIC_ADDRESSES_OUTPUTS: &str = "addresses/{address}/outputs";
pub(crate) const TOPIC_ADDRESSES_ED25519_OUTPUT: &str = "addresses/ed25519/{address}/outputs";

pub struct MqttBrokerConfig {
    pub milestones_latest_tx: LinkTx,
    pub milestones_confirmed_tx: LinkTx,
    pub messages_tx: LinkTx,
    pub messages_referenced_tx: LinkTx,
}

#[derive(Default)]
pub struct MqttBroker;

#[async_trait]
impl<N: Node> Worker<N> for MqttBroker {
    type Config = MqttBrokerConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let MqttBrokerConfig {
            milestones_latest_tx,
            milestones_confirmed_tx,
            messages_tx,
            messages_referenced_tx,
        } = config;

        spawn_static_topic_handler(
            node,
            milestones_latest_tx,
            TOPIC_MILESTONES_LATEST,
            |event: LatestMilestoneChanged| {
                // MilestonePayload as JSON
                let ms_payload_json = serde_json::to_string(&MilestonePayload {
                    index: *event.index,
                    timestamp: event.milestone.timestamp(),
                })
                .expect("error serializing to json");

                (TOPIC_MILESTONES_LATEST, ms_payload_json)
            },
        );

        spawn_static_topic_handler(
            node,
            milestones_confirmed_tx,
            TOPIC_MILESTONES_CONFIRMED,
            |event: LatestSolidMilestoneChanged| {
                // MilestonePayload as JSON
                let ms_payload_json = serde_json::to_string(&MilestonePayload {
                    index: *event.index,
                    timestamp: event.milestone.timestamp(),
                })
                .expect("error serializing to json");

                (TOPIC_MILESTONES_CONFIRMED, ms_payload_json)
            },
        );

        spawn_static_topic_handler(node, messages_tx, TOPIC_MESSAGES, |event: MessageProcessed| {
            // Message in byte-serialized form
            let msg_bytes = event.1;
            (TOPIC_MESSAGES, msg_bytes)
        });

        spawn_static_topic_handler(
            node,
            messages_referenced_tx,
            TOPIC_MESSAGES_REFERENCED,
            |event: MessageConfirmed| {
                // MessageMetadata as JSON
                let msg_metadata_json = serde_json::to_string(&MessageMetadata {
                    message_id: event.message_id.to_string(),
                    parent_message_ids: event.parents.iter().map(|msg_id| msg_id.to_string()).collect(),
                    is_solid: event.is_solid,
                    referenced_by_milestone_index: *event.milestone_index,
                    // TODO: set proper ledger inclusion state
                    ledger_inclusion_state: LedgerInclusionState::NoTransaction,
                    should_promote: false,
                    should_reattach: false,
                })
                .expect("error serializing to json");

                (TOPIC_MESSAGES_REFERENCED, msg_metadata_json)
            },
        );

        // spawn_dynamic_topic_handler(node, "indexation", |event: IndexationMessage| {
        //     let index = hex::encode(event.index);
        //     let bytes = event.bytes;

        //     (&format!("{}/{}", TOPIC_MESSAGES_INDEXATION, index), bytes)
        // });

        info!("MQTT broker started.");

        Ok(Self::default())
    }
}

fn spawn_static_topic_handler<N: Node, E, T, P, F>(node: &mut N, mut tx: LinkTx, topic: &'static str, f: F)
where
    E: Any + Clone + Send + Sync,
    T: Into<String> + Send,
    P: Into<Vec<u8>> + Send,
    F: Fn(E) -> (T, P) + Send + Sync + 'static,
{
    let event_bus = node.bus();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT '{}' topic handler running.", topic);

        let mut events = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(event_rx));

        while let Some(event) = events.next().await {
            let (topic, payload) = f(event);
            if let Err(e) = tx.publish(topic, false, payload) {
                warn!("Publishing MQTT message failed. Cause: {:?}", e);
            }
        }

        debug!("MQTT '{}' topic handler stopped.", topic);
    });

    event_bus.add_listener::<MqttBroker, _, _>(move |event: &E| {
        if event_tx.send(event.clone()).is_err() {
            warn!("Sending event to MQTT '{}' topic handler failed.", topic)
        }
    });
}

// fn spawn_dynamic_topic_handler<N: Node, E, T, P, F>(node: &mut N, topic: &'static str, f: F)
// where
//     E: Any + Clone + Send + Sync,
//     T: Into<String> + Send,
//     P: Into<Vec<u8>> + Send,
//     F: Fn(E) -> (T, P) + Send + Sync + 'static,
// {
//     let event_bus = node.bus();
//     let (event_tx, event_rx) = mpsc::unbounded_channel();

//     node.spawn::<MqttBroker, _, _>(|shutdown| async move {
//         debug!("MQTT '{}' topic handler running.", topic);

//         let mut events = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(event_rx));

//         while let Some(event) = events.next().await {
//             let (topic, payload) = f(event);

//             if let Some(tx) = senders.entry(topic).or_insert(broker.link(&format!("{}/{}", topic,))) {}

//             if let Err(e) = tx.publish(topic, false, payload) {
//                 warn!("Publishing MQTT message failed. Cause: {:?}", e);
//             }
//         }

//         debug!("MQTT '{}' topic handler stopped.", topic);
//     });

//     event_bus.add_listener::<MqttBroker, _, _>(move |event: &E| {
//         if event_tx.send(event.clone()).is_err() {
//             warn!("Sending event to MQTT '{}' topic handler failed.", topic)
//         }
//     });
// }
