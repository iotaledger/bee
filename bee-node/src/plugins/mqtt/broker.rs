// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::event::{LatestMilestoneChanged, LatestSolidMilestoneChanged};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

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
pub(crate) const _TOPIC_MESSAGES: &str = "messages";
pub(crate) const _TOPIC_MESSAGES_REFERENCED: &str = "messages/referenced";
pub(crate) const _TOPIC_MESSAGES_INDEXATION: &str = "messages/indexation/{index}";
pub(crate) const _TOPIC_MESSAGES_METADATA: &str = "messages/{messageId}/metadata";
pub(crate) const _TOPIC_OUTPUTS: &str = "outputs/{outputId}";
pub(crate) const _TOPIC_ADDRESSES_OUTPUTS: &str = "addresses/{address}/outputs";
pub(crate) const _TOPIC_ADDRESSES_ED25519_OUTPUT: &str = "addresses/ed25519/{address}/outputs";

pub struct MqttBrokerConfig {
    pub latest_tx: LinkTx,
    pub confirmed_tx: LinkTx,
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
            latest_tx,
            confirmed_tx,
        } = config;

        spawn_topic_handler(
            node,
            latest_tx,
            TOPIC_MILESTONES_LATEST,
            |event: LatestMilestoneChanged| (TOPIC_MILESTONES_LATEST, format!("{}", event.index)),
        );

        spawn_topic_handler(
            node,
            confirmed_tx,
            TOPIC_MILESTONES_CONFIRMED,
            |event: LatestSolidMilestoneChanged| (TOPIC_MILESTONES_CONFIRMED, format!("{}", event.index)),
        );

        info!("MQTT broker started.");

        Ok(Self::default())
    }
}

fn spawn_topic_handler<N: Node, E, T, P, F>(node: &mut N, mut tx: LinkTx, topic: &'static str, f: F)
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
