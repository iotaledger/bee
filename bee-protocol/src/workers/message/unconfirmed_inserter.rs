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
use bee_tangle::unconfirmed_message::UnconfirmedMessage;

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

const UNCONFIRMED_MESSAGE_BATCH_SIZE: usize = 1000;

pub struct UnconfirmedMessageInserterWorkerEvent(pub(crate) MessageId, pub(crate) MilestoneIndex);

pub struct UnconfirmedMessageInserterWorker {
    pub tx: mpsc::UnboundedSender<UnconfirmedMessageInserterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for UnconfirmedMessageInserterWorker
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

            while let Some(UnconfirmedMessageInserterWorkerEvent(message_id, index)) = receiver.next().await {
                if let Err(e) = Batch::<(MilestoneIndex, UnconfirmedMessage), ()>::batch_insert(
                    &*storage,
                    &mut batch,
                    &(index, UnconfirmedMessage::from(message_id)),
                    &(),
                ) {
                    error!("Batch inserting unconfirmed message failed: {:?}.", e);
                }

                counter += 1;
                if counter == UNCONFIRMED_MESSAGE_BATCH_SIZE {
                    if let Err(e) = storage.batch_commit(batch, true).await {
                        error!("Committing unconfirmed message batch failed: {:?}.", e);
                    }
                    batch = N::Backend::batch_begin();
                    counter = 0;
                }
            }

            if let Err(e) = storage.batch_commit(batch, true).await {
                error!("Committing unconfirmed message batch failed: {:?}.", e);
            }

            // Before the worker completely stops, the receiver needs to be drained for unconfirmed messages to be
            // inserted. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            counter = 0;

            while let Some(Some(UnconfirmedMessageInserterWorkerEvent(message_id, index))) =
                receiver.next().now_or_never()
            {
                if let Err(e) = Insert::<(MilestoneIndex, UnconfirmedMessage), ()>::insert(
                    &*storage,
                    &(index, UnconfirmedMessage::from(message_id)),
                    &(),
                )
                .await
                {
                    error!("Inserting unconfirmed message failed: {:?}.", e);
                }
                counter += 1;
            }

            debug!("Drained {} messages.", counter);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
