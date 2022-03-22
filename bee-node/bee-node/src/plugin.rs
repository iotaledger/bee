// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod inx {
    tonic::include_proto!("inx");
}

use std::pin::Pin;

use self::inx::inx_server::Inx;

use bee_ledger::types::LedgerIndex;
use bee_message::milestone::MilestoneIndex;
use bee_protocol::workers::storage::StorageBackend;
use bee_runtime::{node::Node, resource::ResourceHandle};
use bee_storage::{access::Fetch, system::StorageHealth};
use bee_tangle::Tangle;

use tonic::{Request, Response, Status};

struct PluginServer<B> {
    tangle: ResourceHandle<Tangle<B>>,
    storage: ResourceHandle<B>,
}

impl<B: StorageBackend> PluginServer<B> {
    fn new<N: Node<Backend = B>>(node: N) -> Self {
        Self {
            tangle: node.resource(),
            storage: node.storage(),
        }
    }

    async fn get_milestone(&self, milestone_index: u32) -> inx::Milestone {
        let milestone = self
            .tangle
            .get_milestone(MilestoneIndex(milestone_index))
            .await
            .unwrap();

        inx::Milestone {
            milestone_index,
            // FIXME: unwrap
            milestone_timestamp: milestone.timestamp().try_into().unwrap(),
            message_id: Some(inx::MessageId {
                id: milestone.message_id().as_ref().to_vec(),
            }),
        }
    }
}

trait Stream<T>: futures::Stream<Item = Result<T, Status>> + Sync + Send + 'static {}

impl<T, S: futures::Stream<Item = Result<T, Status>> + Sync + Send + 'static> Stream<T> for S {}

type InxStream<T> = Pin<Box<dyn Stream<T>>>;

#[tonic::async_trait]
impl<B: StorageBackend> Inx for PluginServer<B> {
    async fn read_node_status(&self, request: Request<inx::NoParams>) -> Result<Response<inx::NodeStatus>, Status> {
        let inx::NoParams {} = request.into_inner();

        Ok(Response::new(inx::NodeStatus {
            // // FIXME: unwrap
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
        request: Request<inx::NoParams>,
    ) -> Result<Response<inx::ProtocolParameters>, Status> {
        todo!()
    }

    async fn read_milestone(
        &self,
        request: Request<inx::MilestoneRequest>,
    ) -> Result<Response<inx::Milestone>, Status> {
        let inx::MilestoneRequest { milestone_index } = request.into_inner();

        Ok(Response::new(self.get_milestone(milestone_index).await))
    }

    type ListenToLatestMilestoneStream = InxStream<inx::Milestone>;

    async fn listen_to_latest_milestone(
        &self,
        request: Request<inx::NoParams>,
    ) -> Result<Response<Self::ListenToLatestMilestoneStream>, Status> {
        todo!()
    }

    type ListenToConfirmedMilestoneStream = InxStream<inx::Milestone>;

    async fn listen_to_confirmed_milestone(
        &self,
        request: Request<inx::NoParams>,
    ) -> Result<Response<Self::ListenToConfirmedMilestoneStream>, Status> {
        todo!()
    }

    type ListenToMessagesStream = InxStream<inx::Message>;

    async fn listen_to_messages(
        &self,
        request: Request<inx::MessageFilter>,
    ) -> Result<Response<Self::ListenToMessagesStream>, Status> {
        todo!()
    }

    type ListenToSolidMessagesStream = InxStream<inx::MessageMetadata>;

    async fn listen_to_solid_messages(
        &self,
        request: Request<inx::MessageFilter>,
    ) -> Result<Response<Self::ListenToSolidMessagesStream>, Status> {
        todo!()
    }

    type ListenToReferencedMessagesStream = InxStream<inx::MessageMetadata>;

    async fn listen_to_referenced_messages(
        &self,
        request: Request<inx::MessageFilter>,
    ) -> Result<Response<Self::ListenToReferencedMessagesStream>, Status> {
        todo!()
    }

    async fn submit_message(&self, request: Request<inx::RawMessage>) -> Result<Response<inx::MessageId>, Status> {
        todo!()
    }

    async fn read_message(&self, request: Request<inx::MessageId>) -> Result<Response<inx::RawMessage>, Status> {
        todo!()
    }

    async fn read_message_metadata(
        &self,
        request: Request<inx::MessageId>,
    ) -> Result<Response<inx::MessageMetadata>, Status> {
        todo!()
    }

    type ReadUnspentOutputsStream = InxStream<inx::UnspentOutput>;

    async fn read_unspent_outputs(
        &self,
        request: Request<inx::NoParams>,
    ) -> Result<Response<Self::ReadUnspentOutputsStream>, Status> {
        todo!()
    }

    type ListenToLedgerUpdatesStream = InxStream<inx::LedgerUpdate>;

    async fn listen_to_ledger_updates(
        &self,
        request: Request<inx::LedgerUpdateRequest>,
    ) -> Result<Response<Self::ListenToLedgerUpdatesStream>, Status> {
        todo!()
    }

    async fn read_output(&self, request: Request<inx::OutputId>) -> Result<Response<inx::OutputResponse>, Status> {
        todo!()
    }

    type ListenToMigrationReceiptsStream = InxStream<inx::RawReceipt>;

    async fn listen_to_migration_receipts(
        &self,
        request: Request<inx::NoParams>,
    ) -> Result<Response<Self::ListenToMigrationReceiptsStream>, Status> {
        todo!()
    }

    async fn register_api_route(
        &self,
        request: Request<inx::ApiRouteRequest>,
    ) -> Result<Response<inx::NoParams>, Status> {
        todo!()
    }

    async fn unregister_api_route(
        &self,
        request: Request<inx::ApiRouteRequest>,
    ) -> Result<Response<inx::NoParams>, Status> {
        todo!()
    }

    async fn perform_api_request(
        &self,
        request: Request<inx::ApiRequest>,
    ) -> Result<Response<inx::ApiResponse>, Status> {
        todo!()
    }
}
