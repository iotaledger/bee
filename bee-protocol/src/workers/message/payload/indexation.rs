// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_message::{
    payload::{transaction::Essence, Payload},
    Message, MessageId,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_storage::backend::StorageBackendExt;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    workers::{storage::StorageBackend, MetricsWorker},
};

pub(crate) struct IndexationPayloadWorkerEvent {
    pub(crate) message_id: MessageId,
    pub(crate) message: Message,
}

pub(crate) struct IndexationPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
}

#[cfg_attr(feature = "trace", trace_tools::observe)]
fn process<B: StorageBackend>(storage: &B, metrics: &NodeMetrics, message_id: MessageId, message: Message) {
    let indexation = match message.payload() {
        Some(Payload::Indexation(indexation)) => indexation,
        Some(Payload::Transaction(transaction)) => {
            let Essence::Regular(essence) = transaction.essence();

            if let Some(Payload::Indexation(indexation)) = essence.payload() {
                indexation
            } else {
                error!(
                    "Missing or invalid payload for message {}: expected indexation payload.",
                    message_id
                );
                return;
            }
        }
        _ => {
            error!(
                "Missing or invalid payload for message {}: expected indexation payload.",
                message_id
            );
            return;
        }
    };

    metrics.indexation_payloads_inc(1);

    if let Err(e) = storage.insert(&(indexation.padded_index(), message_id), &()) {
        error!(
            "Inserting indexation payload for message {} failed: {:?}.",
            message_id, e
        );
    }
}

#[async_trait]
impl<N> Worker<N> for IndexationPayloadWorker
where
    N: Node,
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MetricsWorker>()].leak()
    }

    #[cfg_attr(feature = "trace", trace_tools::observe)]
    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let storage = node.storage();
        let metrics = node.resource::<NodeMetrics>();
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(IndexationPayloadWorkerEvent { message_id, message }) = receiver.next().await {
                process(&*storage, &metrics, message_id, message);
            }

            // Before the worker completely stops, the receiver needs to be drained for indexation payloads to be
            // analysed. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(IndexationPayloadWorkerEvent { message_id, message })) = receiver.next().now_or_never()
            {
                process(&*storage, &metrics, message_id, message);
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
