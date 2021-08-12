// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::StorageBackend,
};

use bee_ledger::workers::event::{MilestoneConfirmed, MessageReferenced};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use bee_protocol::types::metrics::NodeMetrics;
use futures::StreamExt;
use log::{debug, error, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::plugins::mqtt::broker::MqttBroker;
use librumqttd as mqtt;
use mqtt::LinkTx;
use bee_tangle::{MsTangle, ConflictReason};
use bee_message::payload::Payload;
use bee_rest_api::types::dtos::LedgerInclusionStateDto;
use bee_rest_api::types::responses::MessageMetadataResponse;

pub(crate) fn spawn<N>(
    node: &mut N,
    mut messages_referenced_tx: LinkTx,
)
    where
        N: Node,
        N::Backend: StorageBackend,
{

    let tangle = node.resource::<MsTangle<N::Backend>>();
    let bus = node.bus();
    let (tx, rx) = mpsc::unbounded_channel::<MessageReferenced>();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT 'message/referenced' topic handler running.");

        let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

        while let Some(event) = receiver.next().await {
            match tangle.get(&event.message_id).await.map(|m| (*m).clone()) {
                Some(message) => {
                    // existing message <=> existing metadata, therefore unwrap() is safe
                    let metadata = tangle.get_metadata(&event.message_id).await.unwrap();

                    // TODO: access constants from URTS
                    let ymrsi_delta = 8;
                    let omrsi_delta = 13;
                    let below_max_depth = 15;

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

                            if metadata.flags().is_milestone() {
                                milestone_index = Some(*milestone);
                            } else {
                                milestone_index = None;
                            }

                            ledger_inclusion_state = Some(if let Some(Payload::Transaction(_)) = message.payload() {
                                if metadata.conflict() != ConflictReason::None {
                                    conflict_reason = Some(metadata.conflict());
                                    LedgerInclusionStateDto::Conflicting
                                } else {
                                    conflict_reason = None;
                                    // maybe not checked by the ledger yet, but still
                                    // returning "included". should
                                    // `metadata.flags().is_conflicting` return an Option
                                    // instead?
                                    LedgerInclusionStateDto::Included
                                }
                            } else {
                                conflict_reason = None;
                                LedgerInclusionStateDto::NoTransaction
                            });
                            should_reattach = None;
                            should_promote = None;
                        } else if metadata.flags().is_solid() {
                            // message is not referenced by a milestone but solid
                            is_solid = true;
                            referenced_by_milestone_index = None;
                            milestone_index = None;
                            ledger_inclusion_state = None;
                            conflict_reason = None;

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
                        } else {
                            // the message is not referenced by a milestone and not solid
                            is_solid = false;
                            referenced_by_milestone_index = None;
                            milestone_index = None;
                            ledger_inclusion_state = None;
                            conflict_reason = None;
                            should_reattach = Some(true);
                            should_promote = Some(false);
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
                        message_id: (&event.message_id).to_string(),
                        parent_message_ids: message.parents().iter().map(|id| id.to_string()).collect(),
                        is_solid,
                        referenced_by_milestone_index,
                        milestone_index,
                        ledger_inclusion_state,
                        conflict_reason: conflict_reason.map(|c| c as u8),
                        should_promote,
                        should_reattach,
                    }).expect("error serializing to json");

                    if let Err(e) = messages_referenced_tx.publish("message/referenced", false, payload) {
                        warn!("Publishing MQTT message failed. Cause: {:?}", e);
                    }
                }
                None => panic!("can not find message"),
            }
        }
        debug!("MQTT 'message/referenced' topic handler stopped.");
    });

    bus.add_listener::<MqttBroker, _, _>(move |event: &MessageReferenced| {
        // The lifetime of the listeners is tied to the lifetime of the Dashboard worker so they are removed together.
        // However, topic handlers are shutdown as soon as the signal is received, causing this send to potentially
        // fail and spam the output. The return is then ignored as not being essential.
        let _ = tx.send((*event).clone());
    });
}