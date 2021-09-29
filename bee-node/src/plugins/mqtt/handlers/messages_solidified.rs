// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::broker::MqttBroker, storage::StorageBackend};

use bee_message::MessageId;
use bee_protocol::workers::event::MessageSolidified;
use bee_rest_api::types::responses::MessageMetadataResponse;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};
use bee_tangle::MsTangle;

use librumqttd::LinkTx;
use log::{debug, warn};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

pub(crate) fn spawn<N>(node: &mut N, mut messages_solidified_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let tangle = node.resource::<MsTangle<N::Backend>>();
    let bus = node.bus();
    let (tx, rx) = mpsc::unbounded_channel::<MessageId>();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT 'message/referenced' topic handler running.");

        let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

        while let Some(message_id) = receiver.next().await {
            // the message is newly referenced and therefore it's safe to unwrap
            let message = tangle.get(&message_id).await.map(|m| (*m).clone()).unwrap();
            // existing message <=> existing metadata, therefore it's safe to unwrap
            let metadata = tangle.get_metadata(&message_id).await.unwrap();

            let (should_promote, should_reattach) = {
                let should_promote;
                let should_reattach;

                // TODO: use constants from URTS
                let ymrsi_delta = 8;
                let omrsi_delta = 13;
                let below_max_depth = 15;

                let lmi = *tangle.get_solid_milestone_index();
                // unwrap() of OMRSI/YMRSI is safe since message is solid
                if (lmi - *metadata.omrsi().unwrap().index()) > below_max_depth {
                    should_promote = Some(false);
                    should_reattach = Some(true);
                } else if (lmi - *metadata.ymrsi().unwrap().index()) > ymrsi_delta || (lmi - omrsi_delta) > omrsi_delta
                {
                    should_promote = Some(true);
                    should_reattach = Some(false);
                } else {
                    should_promote = Some(false);
                    should_reattach = Some(false);
                };

                (should_reattach, should_promote)
            };

            let payload = serde_json::to_string(&MessageMetadataResponse {
                message_id: (&message_id).to_string(),
                parent_message_ids: message.parents().iter().map(|id| id.to_string()).collect(),
                is_solid: true,
                referenced_by_milestone_index: None,
                milestone_index: None,
                ledger_inclusion_state: None,
                conflict_reason: None,
                should_promote,
                should_reattach,
            })
            .expect("error serializing to json");

            if let Err(e) = messages_solidified_tx.publish("messages/metadata", false, payload) {
                warn!("Publishing MQTT message failed. Cause: {:?}", e);
            }

            debug!("MQTT 'messages/metadata' topic handler stopped.");
        }
    });

    // The lifetime of the listeners is tied to the lifetime of the MQTT worker so they are removed
    // together. However, topic handlers are shutdown as soon as the signal is received, causing
    // this send to potentially fail and spam the output. The return is then ignored as not being
    // essential.
    bus.add_listener::<MqttBroker, _, _>(move |event: &MessageSolidified| {
        let _ = tx.send(event.message_id);
    });
}
