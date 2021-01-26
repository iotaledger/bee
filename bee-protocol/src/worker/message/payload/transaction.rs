// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::StorageBackend,
    worker::{IndexationPayloadWorker, IndexationPayloadWorkerEvent, MetricsWorker},
    ProtocolMetrics,
};

use bee_message::{payload::Payload, MessageId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) struct TransactionPayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct TransactionPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
}

async fn process<B: StorageBackend>(
    tangle: &MsTangle<B>,
    metrics: &ProtocolMetrics,
    indexation_payload_worker: &mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
    message_id: MessageId,
) {
    if let Some(message) = tangle.get(&message_id).await.map(|m| (*m).clone()) {
        let transaction = match message.payload() {
            Some(Payload::Transaction(transaction)) => transaction,
            _ => return,
        };

        metrics.transaction_payload_inc(1);

        if let Some(Payload::Indexation(_)) = transaction.essence().payload() {
            if let Err(e) = indexation_payload_worker.send(IndexationPayloadWorkerEvent(message_id)) {
                error!(
                    "Sending message id {} to indexation payload worker failed: {:?}.",
                    message_id, e
                );
            }
        }
    }
}

#[async_trait]
impl<N> Worker<N> for TransactionPayloadWorker
where
    N: Node,
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<IndexationPayloadWorker>(),
            TypeId::of::<MetricsWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let indexation_payload_worker = node.worker::<IndexationPayloadWorker>().unwrap().tx.clone();
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(TransactionPayloadWorkerEvent(message_id)) = receiver.next().await {
                process(&tangle, &metrics, &indexation_payload_worker, message_id).await;
            }

            // Before the worker completely stops, the receiver needs to be drained for transaction payloads to be
            // analysed. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(TransactionPayloadWorkerEvent(message_id))) = receiver.next().now_or_never() {
                process(&tangle, &metrics, &indexation_payload_worker, message_id).await;
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
