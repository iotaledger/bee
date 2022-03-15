// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{storage::StorageBackend, IndexationPayloadWorker, IndexationPayloadWorkerEvent, MetricsWorker},
};

use bee_message::{
    payload::{transaction::Essence, Payload},
    MessageId,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MessageRef;

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct TransactionPayloadWorkerEvent {
    pub(crate) message_id: MessageId,
    pub(crate) message: MessageRef,
}

pub(crate) struct TransactionPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
}

fn process(
    message_id: MessageId,
    message: MessageRef,
    indexation_payload_worker: &mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
    metrics: &NodeMetrics,
) {
    let transaction = if let Some(Payload::Transaction(transaction)) = message.payload() {
        transaction
    } else {
        error!(
            "Missing or invalid payload for message {}: expected transaction payload.",
            message_id
        );
        return;
    };

    metrics.transaction_payloads_inc(1);

    let Essence::Regular(essence) = transaction.essence();

    if let Some(Payload::Indexation(_)) = essence.payload() {
        if indexation_payload_worker
            .send(IndexationPayloadWorkerEvent { message_id, message })
            .is_err()
        {
            error!("Sending message {} to indexation payload worker failed.", message_id);
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
        vec![TypeId::of::<IndexationPayloadWorker>(), TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        // SAFETY: unwrapping is fine because IndexationPayloadWorker is in the dependencies.
        let indexation_payload_worker = node.worker::<IndexationPayloadWorker>().unwrap().tx.clone();
        let metrics = node.resource::<NodeMetrics>();
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(TransactionPayloadWorkerEvent { message_id, message }) = receiver.next().await {
                process(message_id, message, &indexation_payload_worker, &metrics);
            }

            // Before the worker completely stops, the receiver needs to be drained for transaction payloads to be
            // analysed; otherwise, they would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(TransactionPayloadWorkerEvent { message_id, message })) = receiver.next().now_or_never()
            {
                process(message_id, message, &indexation_payload_worker, &metrics);
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
