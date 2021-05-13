// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::workers::storage::StorageBackend;

use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_runtime::{
    node::Node,
    shutdown_stream::ShutdownStream,
    worker::{Error as WorkerError, Worker},
};
use bee_storage::access::{Batch, BatchBuilder, Insert};
use bee_tangle::unreferenced_message::UnreferencedMessage;

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

const UNREFERENCED_MESSAGE_BATCH_SIZE: usize = 1000;

pub struct UnreferencedMessageInserterWorkerEvent(pub(crate) MessageId, pub(crate) MilestoneIndex);

pub struct UnreferencedMessageInserterWorker {
    pub tx: mpsc::UnboundedSender<UnreferencedMessageInserterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for UnreferencedMessageInserterWorker
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

        node.spawn::<Self, _, _>(file!(), line!(), |shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(UnreferencedMessageInserterWorkerEvent(message_id, index)) = receiver.next().await {
                if let Err(e) = Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_insert(
                    &*storage,
                    &mut batch,
                    &(index, UnreferencedMessage::from(message_id)),
                    &(),
                ) {
                    error!("Batch inserting unreferenced message failed: {:?}.", e);
                }

                counter += 1;
                if counter == UNREFERENCED_MESSAGE_BATCH_SIZE {
                    if let Err(e) = storage.batch_commit(batch, true).await {
                        error!("Committing unreferenced message batch failed: {:?}.", e);
                    }
                    batch = N::Backend::batch_begin();
                    counter = 0;
                }
            }

            if let Err(e) = storage.batch_commit(batch, true).await {
                error!("Committing unreferenced message batch failed: {:?}.", e);
            }

            // Before the worker completely stops, the receiver needs to be drained for unreferenced messages to be
            // inserted. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            counter = 0;

            while let Some(Some(UnreferencedMessageInserterWorkerEvent(message_id, index))) =
                receiver.next().now_or_never()
            {
                if let Err(e) = Insert::<(MilestoneIndex, UnreferencedMessage), ()>::insert(
                    &*storage,
                    &(index, UnreferencedMessage::from(message_id)),
                    &(),
                )
                .await
                {
                    error!("Inserting unreferenced message failed: {:?}.", e);
                }
                counter += 1;
            }

            debug!("Drained {} messages.", counter);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
