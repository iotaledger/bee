// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::broker::MqttBroker, storage::StorageBackend};

use bee_ledger::workers::event::MessageReferenced;
use bee_message::{payload::Payload, MessageId};
use bee_rest_api::types::{dtos::LedgerInclusionStateDto, responses::MessageMetadataResponse};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};
use bee_tangle::{ConflictReason, MsTangle};

use bee_protocol::workers::event::MessageSolidified;
use librumqttd::LinkTx;
use log::{debug, warn};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

pub(crate) fn spawn<N>(node: &mut N, mut messages_metadata_tx: LinkTx)
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

            let (
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason,
                should_promote,
                should_reattach,
            ) = {
                let is_solid;
                let referenced_by_milestone_index;
                let milestone_index;
                let ledger_inclusion_state;
                let conflict_reason;
                let should_promote;
                let should_reattach;

                if let Some(milestone) = metadata.milestone_index() {
                    // message is referenced by a milestone
                    is_solid = true;
                    referenced_by_milestone_index = Some(*milestone);
                    should_reattach = None;
                    should_promote = None;

                    // check if the message is an actual milestone
                    if metadata.flags().is_milestone() {
                        milestone_index = Some(*milestone);
                        ledger_inclusion_state = Some(LedgerInclusionStateDto::NoTransaction);
                        conflict_reason = None;
                    } else {
                        milestone_index = None;
                        if let Some(Payload::Transaction(_)) = message.payload() {
                            if metadata.conflict() != ConflictReason::None {
                                ledger_inclusion_state = Some(LedgerInclusionStateDto::Conflicting);
                                conflict_reason = Some(metadata.conflict());
                            } else {
                                ledger_inclusion_state = Some(LedgerInclusionStateDto::Included);
                                conflict_reason = None;
                            }
                        } else {
                            ledger_inclusion_state = Some(LedgerInclusionStateDto::NoTransaction);
                            conflict_reason = None;
                        };
                    }
                } else {
                    // message is not referenced by a milestone but solid
                    is_solid = true;
                    referenced_by_milestone_index = None;
                    milestone_index = None;
                    ledger_inclusion_state = None;
                    conflict_reason = None;

                    // TODO: use constants from URTS
                    let ymrsi_delta = 8;
                    let omrsi_delta = 13;
                    let below_max_depth = 15;

                    let lmi = *tangle.get_solid_milestone_index();
                    // unwrap() of OMRSI/YMRSI is safe since message is solid
                    if (lmi - *metadata.omrsi().unwrap().index()) > below_max_depth {
                        should_promote = Some(false);
                        should_reattach = Some(true);
                    } else if (lmi - *metadata.ymrsi().unwrap().index()) > ymrsi_delta
                        || (lmi - omrsi_delta) > omrsi_delta
                    {
                        should_promote = Some(true);
                        should_reattach = Some(false);
                    } else {
                        should_promote = Some(false);
                        should_reattach = Some(false);
                    };
                }

                (
                    is_solid,
                    referenced_by_milestone_index,
                    milestone_index,
                    ledger_inclusion_state,
                    conflict_reason,
                    should_reattach,
                    should_promote,
                )
            };

            let payload = serde_json::to_string(&MessageMetadataResponse {
                message_id: (&message_id).to_string(),
                parent_message_ids: message.parents().iter().map(|id| id.to_string()).collect(),
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason: conflict_reason.map(|c| c as u8),
                should_promote,
                should_reattach,
            })
            .expect("error serializing to json");

            if let Err(e) = messages_metadata_tx.publish("messages/metadata", false, payload) {
                warn!("Publishing MQTT message failed. Cause: {:?}", e);
            }

            debug!("MQTT 'messages/metadata' topic handler stopped.");
        }
    });

    // The lifetime of the listeners is tied to the lifetime of the MQTT worker so they are removed
    // together. However, topic handlers are shutdown as soon as the signal is received, causing
    // this send to potentially fail and spam the output. The return is then ignored as not being
    // essential.
    {
        let tx = tx.clone();
        bus.add_listener::<MqttBroker, _, _>(move |event: &MessageReferenced| {
            let _ = tx.send(event.message_id);
        });
    }

    {
        let tx = tx.clone();
        bus.add_listener::<MqttBroker, _, _>(move |event: &MessageSolidified| {
            let _ = tx.send(event.message_id);
        });
    }
}
