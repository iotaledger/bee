// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod config;
mod topics;

use topics::*;

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{
    event::Bus,
    node::{Node, ResHandle},
    worker::Worker,
};
use bee_protocol::event::{LatestMilestoneChanged, LatestSolidMilestoneChanged};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, error, warn};
use paho_mqtt as mqtt;
use tokio::sync::mpsc;

use std::{any::Any, convert::Infallible, time::Duration};

#[derive(Default)]
pub struct Mqtt;

fn send_message<P>(client: &ResHandle<mqtt::AsyncClient>, topic: &'static str, payload: P)
where
    P: Into<Vec<u8>>,
{
    // TODO Send to all that registered to this topic
    if let Err(e) = client.publish(mqtt::Message::new(topic, payload, 0)).wait() {
        warn!("Publishing mqtt message on topic {} failed: {:?}.", topic, e);
    }
}

fn topic_handler<N, E, P, F>(node: &mut N, topic: &'static str, f: F)
where
    N: Node,
    E: Any + Clone + Send,
    P: Into<Vec<u8>>,
    F: 'static + Fn(&E) -> P + Send,
{
    let bus = node.resource::<Bus>();
    let client = node.resource::<mqtt::AsyncClient>();
    let (tx, rx) = mpsc::unbounded_channel();

    node.spawn::<Mqtt, _, _>(|shutdown| async move {
        debug!("Mqtt {} topic handler running.", topic);

        let mut receiver = ShutdownStream::new(shutdown, rx);

        while let Some(event) = receiver.next().await {
            send_message(&client, topic, f(&event));
        }

        debug!("Mqtt {} topic handler stopped.", topic);
    });

    bus.add_listener::<Mqtt, _, _>(move |event: &E| {
        if let Err(_) = tx.send((*event).clone()) {
            warn!("Sending event to mqtt {} topic handler failed.", topic)
        }
    });
}

#[async_trait]
impl<N: Node> Worker<N> for Mqtt {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        // TODO conf
        match mqtt::AsyncClient::new("tcp://localhost:1883") {
            Ok(client) => {
                node.register_resource(client);
                let client = node.resource::<mqtt::AsyncClient>();

                let conn_opts = mqtt::ConnectOptionsBuilder::new()
                    .keep_alive_interval(Duration::from_secs(20))
                    .clean_session(true)
                    .finalize();

                match client.connect(conn_opts).wait() {
                    Ok(_) => {
                        // TODO log connected

                        topic_handler(node, TOPIC_MILESTONES_LATEST, |_event: &LatestMilestoneChanged| "");
                        topic_handler(node, TOPIC_MILESTONES_SOLID, |_event: &LatestSolidMilestoneChanged| "");
                        // topic_handler(node, _TOPIC_MESSAGES, |_event: &_| {});
                        // topic_handler(node, _TOPIC_MESSAGES_REFERENCED, |_event: &_| {});
                        // topic_handler(node, _TOPIC_MESSAGES_INDEXATION, |_event: &_| {});
                        // topic_handler(node, _TOPIC_MESSAGES_METADATA, |_event: &_| {});
                        // topic_handler(node, _TOPIC_OUTPUTS, |_event: &_| {});
                        // topic_handler(node, _TOPIC_ADDRESSES_OUTPUTS, |_event: &_| {});
                        // topic_handler(node, _TOPIC_ADDRESSES_ED25519_OUTPUT, |_event: &_| {});
                    }
                    Err(e) => {
                        error!("Connecting mqtt client failed {:?}.", e);
                    }
                }
            }
            Err(e) => {
                error!("Creating mqtt client failed {:?}.", e);
            }
        }

        Ok(Self::default())
    }

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        // if let Some(client) = node.remove_resource::<mqtt::AsyncClient>() {
        //     client.disconnect(None).wait().unwrap();
        // }

        Ok(())
    }
}
