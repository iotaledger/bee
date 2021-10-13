// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::broker::MqttBroker, storage::StorageBackend};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use librumqttd::LinkTx;
use log::{debug, warn};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use std::any::Any;

pub(crate) mod addresses_ed25519_ouptuts_consumed;
pub(crate) mod addresses_ed25519_ouptuts_created;
pub(crate) mod addresses_ouptuts_consumed;
pub(crate) mod addresses_ouptuts_created;
pub(crate) mod messages;
pub(crate) mod messages_indexation;
pub(crate) mod messages_referenced;
pub(crate) mod messages_solidified;
pub(crate) mod milestones_confirmed;
pub(crate) mod milestones_latest;
pub(crate) mod outputs_consumed;
pub(crate) mod outputs_created;

fn spawn_static_topic_handler<N, E, T, P, F>(node: &mut N, mut tx: LinkTx, handler_name: &'static str, into_mqtt: F)
where
    N: Node,
    N::Backend: StorageBackend,
    E: Any + Clone + Send + Sync,
    T: Into<String> + Send,
    P: Into<Vec<u8>> + Send,
    F: Fn(E) -> (T, P) + Send + Sync + 'static,
{
    let bus = node.bus();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT '{}' topic handler running.", handler_name);

        let mut event_rx = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(event_rx));

        while let Some(event) = event_rx.next().await {
            // Turn the received event bound to this handler into MQTT data as specified in the Node Events API.
            let (topic, payload) = into_mqtt(event);

            if let Err(e) = tx.publish(topic, false, payload) {
                warn!("Publishing MQTT message failed. Cause: {:?}", e);
            }
        }

        debug!("MQTT '{}' topic handler stopped.", handler_name);
    });

    bus.add_listener::<MqttBroker, _, _>(move |event: &E| {
        if event_tx.send(event.clone()).is_err() {
            warn!("Sending event to MQTT '{}' topic handler failed.", handler_name)
        }
    });
}