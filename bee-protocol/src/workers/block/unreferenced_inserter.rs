// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use bee_block::{payload::milestone::MilestoneIndex, BlockId};
use bee_runtime::{
    node::Node,
    shutdown_stream::ShutdownStream,
    worker::{Error as WorkerError, Worker},
};
use bee_storage::access::{Batch, BatchBuilder, Insert};
use bee_tangle::unreferenced_block::UnreferencedBlock;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::workers::storage::StorageBackend;

const UNREFERENCED_MESSAGE_BATCH_SIZE: usize = 1000;

pub struct UnreferencedBlockInserterWorkerEvent(pub(crate) BlockId, pub(crate) MilestoneIndex);

pub struct UnreferencedBlockInserterWorker {
    pub tx: mpsc::UnboundedSender<UnreferencedBlockInserterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for UnreferencedBlockInserterWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let storage = node.storage();

        let mut batch = N::Backend::batch_begin();
        let mut counter = 0;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(UnreferencedBlockInserterWorkerEvent(block_id, index)) = receiver.next().await {
                if let Err(e) = Batch::<(MilestoneIndex, UnreferencedBlock), ()>::batch_insert(
                    &*storage,
                    &mut batch,
                    &(index, UnreferencedBlock::from(block_id)),
                    &(),
                ) {
                    error!("Batch inserting unreferenced block failed: {:?}.", e);
                }

                counter += 1;
                if counter == UNREFERENCED_MESSAGE_BATCH_SIZE {
                    if let Err(e) = storage.batch_commit(batch, true) {
                        error!("Committing unreferenced block batch failed: {:?}.", e);
                    }
                    batch = N::Backend::batch_begin();
                    counter = 0;
                }
            }

            if let Err(e) = storage.batch_commit(batch, true) {
                error!("Committing unreferenced block batch failed: {:?}.", e);
            }

            // Before the worker completely stops, the receiver needs to be drained for unreferenced blocks to be
            // inserted. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            counter = 0;

            while let Some(Some(UnreferencedBlockInserterWorkerEvent(block_id, index))) = receiver.next().now_or_never()
            {
                if let Err(e) = Insert::<(MilestoneIndex, UnreferencedBlock), ()>::insert(
                    &*storage,
                    &(index, UnreferencedBlock::from(block_id)),
                    &(),
                ) {
                    error!("Inserting unreferenced block failed: {:?}.", e);
                }
                counter += 1;
            }

            debug!("Drained {} blocks.", counter);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
