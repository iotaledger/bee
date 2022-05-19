// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_block::{
    payload::{transaction::TransactionEssence, Payload},
    Block, BlockId,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    workers::{storage::StorageBackend, MetricsWorker, TaggedDataPayloadWorker, TaggedDataPayloadWorkerEvent},
};

pub(crate) struct TransactionPayloadWorkerEvent {
    pub(crate) block_id: BlockId,
    pub(crate) block: Block,
}

pub(crate) struct TransactionPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
}

fn process(
    block_id: BlockId,
    block: Block,
    tagged_data_payload_worker: &mpsc::UnboundedSender<TaggedDataPayloadWorkerEvent>,
    metrics: &NodeMetrics,
) {
    let transaction = if let Some(Payload::Transaction(transaction)) = block.payload() {
        transaction
    } else {
        error!(
            "Missing or invalid payload for block {}: expected transaction payload.",
            block_id
        );
        return;
    };

    metrics.transaction_payloads_inc(1);

    let TransactionEssence::Regular(essence) = transaction.essence();

    if let Some(Payload::TaggedData(_)) = essence.payload() {
        if tagged_data_payload_worker
            .send(TaggedDataPayloadWorkerEvent {})
            .is_err()
        {
            error!("Sending block {} to tagged data payload worker failed.", block_id);
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
        vec![TypeId::of::<TaggedDataPayloadWorker>(), TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        // SAFETY: unwrapping is fine because TaggedDataPayloadWorker is in the dependencies.
        let tagged_data_payload_worker = node.worker::<TaggedDataPayloadWorker>().unwrap().tx.clone();
        let metrics = node.resource::<NodeMetrics>();
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(TransactionPayloadWorkerEvent { block_id, block }) = receiver.next().await {
                process(block_id, block, &tagged_data_payload_worker, &metrics);
            }

            // Before the worker completely stops, the receiver needs to be drained for transaction payloads to be
            // analysed; otherwise, they would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(TransactionPayloadWorkerEvent { block_id, block })) = receiver.next().now_or_never() {
                process(block_id, block, &tagged_data_payload_worker, &metrics);
                count += 1;
            }

            debug!("Drained {} events.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
