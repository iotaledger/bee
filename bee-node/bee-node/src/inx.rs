// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::pin::Pin;

use bee_ledger::types::{ConsumedOutput, CreatedOutput, LedgerIndex, Unspent};
use bee_ledger::workers::event::LedgerUpdated;
use bee_message::{
    milestone::MilestoneIndex, output::OutputId, payload::Payload, semantic::ConflictReason, Message, MessageId,
};
use bee_protocol::workers::{
    event::{MessageProcessed, MessageSolidified},
    MessageSubmitterError, MessageSubmitterWorker, MessageSubmitterWorkerEvent,
};
use bee_runtime::{event::Bus, node::Node, resource::ResourceHandle};
use bee_storage::{
    access::{Exist, Fetch},
    system::StorageHealth,
};
use bee_tangle::{
    event::{ConfirmedMilestoneChanged, LatestMilestoneChanged},
    metadata::MessageMetadata,
    Tangle,
};
use inx::{proto, server::Inx, Request, Response, Status};
use packable::PackableExt;

pub trait StorageBackend:
    Fetch<OutputId, ConsumedOutput>
    + bee_storage::backend::StorageBackend
    + bee_protocol::workers::storage::StorageBackend
    + bee_ledger::workers::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: Fetch<OutputId, ConsumedOutput>
        + bee_storage::backend::StorageBackend
        + bee_protocol::workers::storage::StorageBackend
        + bee_ledger::workers::storage::StorageBackend
{
}

struct PluginServer<B> {
    tangle: ResourceHandle<Tangle<B>>,
    storage: ResourceHandle<B>,
    bus: ResourceHandle<Bus<'static>>,
    message_submitter: MessageSubmitterWorker,
}

impl<B: StorageBackend> PluginServer<B> {
    fn new<N: Node<Backend = B>>(node: N) -> Self {
        Self {
            tangle: node.resource(),
            storage: node.storage(),
            bus: node.bus(),
            // FIXME: unwrap
            message_submitter: node.worker::<MessageSubmitterWorker>().unwrap().clone(),
        }
    }

    async fn get_milestone(&self, milestone_index: u32) -> proto::Milestone {
        let milestone = self
            .tangle
            .get_milestone(MilestoneIndex(milestone_index))
            .await
            .unwrap();

        proto::Milestone {
            milestone_index,
            // FIXME: unwrap
            milestone_timestamp: milestone.timestamp().try_into().unwrap(),
            message_id: Some(proto::MessageId {
                id: milestone.message_id().as_ref().to_vec(),
            }),
        }
    }

    fn get_metadata(
        tangle: &Tangle<B>,
        storage: &B,
        message_id: &MessageId,
        parents: &[MessageId],
    ) -> proto::MessageMetadata {
        // FIXME: unwrap
        let message = Fetch::<MessageId, Message>::fetch(storage, message_id)
            .unwrap()
            .unwrap();
        // FIXME: unwrap
        let metadata = Fetch::<MessageId, MessageMetadata>::fetch(storage, message_id)
            .unwrap()
            .unwrap();

        // FIXME: deduplicate logic from bee-api
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
                        2 // Conflicting
                    } else {
                        conflict_reason = None;
                        // maybe not checked by the ledger yet, but still
                        // returning "included". should
                        // `metadata.flags().is_conflicting` return an Option
                        // instead?
                        1 // Included
                    }
                } else {
                    conflict_reason = None;
                    0 // No Transaction
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

                let cmi = *tangle.get_confirmed_milestone_index();
                // unwrap() of OMRSI/YMRSI is safe since message is solid
                if (cmi - *metadata.omrsi().unwrap().index()) > below_max_depth {
                    should_promote = Some(false);
                    should_reattach = Some(true);
                } else if (cmi - *metadata.ymrsi().unwrap().index()) > ymrsi_delta
                    || (cmi - *metadata.omrsi().unwrap().index()) > omrsi_delta
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
                // FIXME: is this consistent?
                conflict_reason,
                should_reattach,
                should_promote,
            )
        };

        proto::MessageMetadata {
            message_id: Some(proto::MessageId {
                id: message_id.as_ref().to_vec(),
            }),
            parents: parents.iter().map(|message_id| message_id.as_ref().to_vec()).collect(),
            solid: is_solid,
            // FIXME: unwrap
            should_promote: should_promote.unwrap_or_default(),
            // FIXME: unwrap
            should_reattach: should_reattach.unwrap_or_default(),
            // FIXME: unwrap
            referenced_by_milestone_index: referenced_by_milestone_index.unwrap_or_default(),
            // FIXME: unwrap
            milestone_index: milestone_index.unwrap_or_default(),
            // FIXME: unwrap
            ledger_inclusion_state: ledger_inclusion_state.unwrap_or_default(),
            // FIXME: unwrap
            conflict_reason: conflict_reason.map(|reason| (reason as u8).into()).unwrap_or_default(),
        }
    }
}

trait Stream<T>: futures::Stream<Item = Result<T, Status>> + Sync + Send + 'static {}

impl<T, S: futures::Stream<Item = Result<T, Status>> + Sync + Send + 'static> Stream<T> for S {}

type InxStream<T> = Pin<Box<dyn Stream<T>>>;

#[async_trait::async_trait]
impl<B: StorageBackend> Inx for PluginServer<B> {
    async fn read_node_status(&self, request: Request<proto::NoParams>) -> Result<Response<proto::NodeStatus>, Status> {
        let proto::NoParams {} = request.into_inner();

        Ok(Response::new(proto::NodeStatus {
            // FIXME: unwrap
            is_healthy: self.storage.get_health().unwrap().unwrap() == StorageHealth::Healthy,
            latest_milestone: Some(self.get_milestone(*self.tangle.get_latest_milestone_index()).await),
            confirmed_milestone: Some(self.get_milestone(*self.tangle.get_confirmed_milestone_index()).await),
            pruning_index: *self.tangle.get_pruning_index(),
            // FIXME: unwrap
            ledger_index: *Fetch::<(), LedgerIndex>::fetch(&*self.storage, &()).unwrap().unwrap(),
        }))
    }

    async fn read_protocol_parameters(
        &self,
        request: Request<proto::NoParams>,
    ) -> Result<Response<proto::ProtocolParameters>, Status> {
        let proto::NoParams {} = request.into_inner();

        todo!()
    }

    async fn read_milestone(
        &self,
        request: Request<proto::MilestoneRequest>,
    ) -> Result<Response<proto::Milestone>, Status> {
        let proto::MilestoneRequest { milestone_index } = request.into_inner();

        Ok(Response::new(self.get_milestone(milestone_index).await))
    }

    // FIXME: avoid dynamic dispatch
    type ListenToLatestMilestoneStream = InxStream<proto::Milestone>;

    async fn listen_to_latest_milestone(
        &self,
        request: Request<proto::NoParams>,
    ) -> Result<Response<Self::ListenToLatestMilestoneStream>, Status> {
        let proto::NoParams {} = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // FIXME: `TypeId` might not be good enough
        self.bus.add_listener::<Self, LatestMilestoneChanged, _>(move |event| {
            tx.send(Ok(proto::Milestone {
                milestone_index: *event.index,
                // FIXME: unwrap
                milestone_timestamp: event.milestone.timestamp().try_into().unwrap(),
                message_id: Some(proto::MessageId {
                    id: event.milestone.message_id().as_ref().to_vec(),
                }),
            }))
            // FIXME: unwrap
            .unwrap();
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        )))
    }

    // FIXME: avoid dynamic dispatch
    type ListenToConfirmedMilestoneStream = InxStream<proto::Milestone>;

    async fn listen_to_confirmed_milestone(
        &self,
        request: Request<proto::NoParams>,
    ) -> Result<Response<Self::ListenToConfirmedMilestoneStream>, Status> {
        let proto::NoParams {} = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // FIXME: `TypeId` might not be good enough
        self.bus
            .add_listener::<Self, ConfirmedMilestoneChanged, _>(move |event| {
                tx.send(Ok(proto::Milestone {
                    milestone_index: *event.index,
                    // FIXME: unwrap
                    milestone_timestamp: event.milestone.timestamp().try_into().unwrap(),
                    message_id: Some(proto::MessageId {
                        id: event.milestone.message_id().as_ref().to_vec(),
                    }),
                }))
                // FIXME: unwrap
                .unwrap();
            });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        )))
    }

    type ListenToMessagesStream = InxStream<proto::Message>;

    async fn listen_to_messages(
        &self,
        request: Request<proto::MessageFilter>,
    ) -> Result<Response<Self::ListenToMessagesStream>, Status> {
        let proto::MessageFilter {} = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // FIXME: `TypeId` might not be good enough
        self.bus.add_listener::<Self, MessageProcessed, _>(move |event| {
            tx.send(Ok(proto::Message {
                message_id: Some(proto::MessageId {
                    id: event.message_id.as_ref().to_vec(),
                }),
                message: Some(proto::RawMessage {
                    data: event.bytes.clone(),
                }),
            }))
            // FIXME: unwrap
            .unwrap();
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        )))
    }

    type ListenToSolidMessagesStream = InxStream<proto::MessageMetadata>;

    async fn listen_to_solid_messages(
        &self,
        request: Request<proto::MessageFilter>,
    ) -> Result<Response<Self::ListenToSolidMessagesStream>, Status> {
        let proto::MessageFilter {} = request.into_inner();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let storage = self.storage.clone();
        let tangle = self.tangle.clone();
        // FIXME: `TypeId` might not be good enough
        self.bus.add_listener::<Self, MessageSolidified, _>(move |event| {
            tx.send(Ok(Self::get_metadata(
                &*tangle,
                &*storage,
                &event.message_id,
                &event.parents,
            )))
            // FIXME: unwrap
            .unwrap();
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        )))
    }

    type ListenToReferencedMessagesStream = InxStream<proto::MessageMetadata>;

    async fn listen_to_referenced_messages(
        &self,
        request: Request<proto::MessageFilter>,
    ) -> Result<Response<Self::ListenToReferencedMessagesStream>, Status> {
        let proto::MessageFilter {} = request.into_inner();

        todo!()
    }

    async fn submit_message(&self, request: Request<proto::RawMessage>) -> Result<Response<proto::MessageId>, Status> {
        let proto::RawMessage { data: message } = request.into_inner();

        let (notifier, waiter) = futures::channel::oneshot::channel::<Result<MessageId, MessageSubmitterError>>();
        // FIXME: unwrap
        self.message_submitter
            .tx
            .send(MessageSubmitterWorkerEvent { message, notifier })
            .ok()
            .unwrap();

        Ok(Response::new(proto::MessageId {
            // FIXME: unwrap
            id: waiter.await.unwrap().unwrap().as_ref().to_vec(),
        }))
    }

    async fn read_message(&self, request: Request<proto::MessageId>) -> Result<Response<proto::RawMessage>, Status> {
        let proto::MessageId { id: bytes } = request.into_inner();
        // FIXME: unwrap
        let message_id = MessageId::new(bytes.try_into().unwrap());

        Ok(Response::new(proto::RawMessage {
            // FIXME: unwrap
            data: self.tangle.get(&message_id).await.unwrap().pack_to_vec(),
        }))
    }

    async fn read_message_metadata(
        &self,
        request: Request<proto::MessageId>,
    ) -> Result<Response<proto::MessageMetadata>, Status> {
        let proto::MessageId { id: bytes } = request.into_inner();
        // FIXME: unwrap
        let message_id = MessageId::new(bytes.try_into().unwrap());
        // FIXME: unwrap
        let parents = Fetch::<MessageId, Vec<MessageId>>::fetch(&*self.storage, &message_id)
            .unwrap()
            .unwrap();

        Ok(Response::new(Self::get_metadata(
            &*self.tangle,
            &*self.storage,
            &message_id,
            &parents,
        )))
    }

    type ReadUnspentOutputsStream = InxStream<proto::UnspentOutput>;

    async fn read_unspent_outputs(
        &self,
        request: Request<proto::NoParams>,
    ) -> Result<Response<Self::ReadUnspentOutputsStream>, Status> {
        let proto::NoParams {} = request.into_inner();

        todo!()
    }

    type ListenToLedgerUpdatesStream = InxStream<proto::LedgerUpdate>;

    async fn listen_to_ledger_updates(
        &self,
        request: Request<proto::LedgerUpdateRequest>,
    ) -> Result<Response<Self::ListenToLedgerUpdatesStream>, Status> {
        let proto::LedgerUpdateRequest { start_milestone_index } = request.into_inner();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // FIXME: `TypeId` might not be good enough
        self.bus.add_listener::<Self, LedgerUpdated, _>(move |event| {
            let milestone_index = *event.milestone_index;
            if milestone_index >= start_milestone_index {
                tx.send(Ok(proto::LedgerUpdate {
                    milestone_index,
                    consumed: event
                        .consumed_outputs
                        .iter()
                        .map(|(output_id, (created_output, consumed_output))| proto::LedgerSpent {
                            output: Some(proto::LedgerOutput {
                                output_id: Some(proto::OutputId {
                                    id: output_id.pack_to_vec(),
                                }),
                                message_id: Some(proto::MessageId {
                                    id: created_output.message_id().as_ref().to_vec(),
                                }),
                                milestone_index_booked: *created_output.milestone_index(),
                                milestone_timestamp_booked: created_output.milestone_timestamp(),
                                output: created_output.inner().pack_to_vec(),
                            }),
                            milestone_index_spent: *consumed_output.milestone_index(),
                            milestone_timestamp_spent: consumed_output.milestone_timestamp(),
                            transaction_id_spent: consumed_output.target().pack_to_vec(),
                        })
                        .collect(),
                    created: event
                        .created_outputs
                        .iter()
                        .map(|(output_id, created_output)| proto::LedgerOutput {
                            output_id: Some(proto::OutputId {
                                id: output_id.pack_to_vec(),
                            }),
                            message_id: Some(proto::MessageId {
                                id: created_output.message_id().as_ref().to_vec(),
                            }),
                            milestone_index_booked: *created_output.milestone_index(),
                            milestone_timestamp_booked: created_output.milestone_timestamp(),
                            output: created_output.inner().pack_to_vec(),
                        })
                        .collect(),
                }))
                // FIXME: unwrap
                .unwrap();
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        )))
    }

    async fn read_output(&self, request: Request<proto::OutputId>) -> Result<Response<proto::OutputResponse>, Status> {
        let proto::OutputId { id: bytes } = request.into_inner();
        // FIXME: unwrap
        let output_id = OutputId::unpack_verified(&bytes).unwrap();

        // FIXME: unwrap
        let output = Fetch::<OutputId, CreatedOutput>::fetch(&*self.storage, &output_id)
            .unwrap()
            .unwrap();

        // FIXME: unwrap
        let is_spent = !Exist::<Unspent, ()>::exist(&*self.storage, &Unspent::from(output_id)).unwrap();

        // FIXME: unwrap
        let ledger_index = Fetch::<(), LedgerIndex>::fetch(&*self.storage, &()).unwrap().unwrap();

        let created_output = proto::LedgerOutput {
            output_id: Some(proto::OutputId { id: bytes }),
            message_id: Some(proto::MessageId {
                id: output.message_id().as_ref().to_vec(),
            }),
            // TODO
            milestone_index_booked: 0,
            // TODO
            milestone_timestamp_booked: 0,
            output: output.pack_to_vec(),
        };

        let payload = if is_spent {
            // FIXME: unwrap
            let consumed_output = Fetch::<OutputId, ConsumedOutput>::fetch(&*self.storage, &output_id)
                .unwrap()
                .unwrap();

            proto::output_response::Payload::Spent(proto::LedgerSpent {
                output: Some(created_output),
                transaction_id_spent: consumed_output.target().pack_to_vec(),
                milestone_index_spent: *consumed_output.milestone_index(),
                milestone_timestamp_spent: consumed_output.milestone_timestamp(),
            })
        } else {
            proto::output_response::Payload::Output(created_output)
        };

        Ok(Response::new(proto::OutputResponse {
            ledger_index: *ledger_index,
            payload: Some(payload),
        }))
    }

    type ListenToMigrationReceiptsStream = InxStream<proto::RawReceipt>;

    async fn listen_to_migration_receipts(
        &self,
        request: Request<proto::NoParams>,
    ) -> Result<Response<Self::ListenToMigrationReceiptsStream>, Status> {
        todo!()
    }

    async fn register_api_route(
        &self,
        request: Request<proto::ApiRouteRequest>,
    ) -> Result<Response<proto::NoParams>, Status> {
        todo!()
    }

    async fn unregister_api_route(
        &self,
        request: Request<proto::ApiRouteRequest>,
    ) -> Result<Response<proto::NoParams>, Status> {
        todo!()
    }

    async fn perform_api_request(
        &self,
        request: Request<proto::ApiRequest>,
    ) -> Result<Response<proto::ApiResponse>, Status> {
        todo!()
    }
}
