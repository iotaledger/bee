// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod milestone;
mod tagged_data;
mod transaction;

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_block::{payload::Payload, Block, BlockId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub(crate) use self::{
    milestone::{MilestonePayloadWorker, MilestonePayloadWorkerEvent},
    tagged_data::{TaggedDataPayloadWorker, TaggedDataPayloadWorkerEvent},
    transaction::{TransactionPayloadWorker, TransactionPayloadWorkerEvent},
};
use crate::workers::storage::StorageBackend;

pub(crate) struct PayloadWorkerEvent {
    pub(crate) block_id: BlockId,
    pub(crate) block: Block,
}

pub(crate) struct PayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<PayloadWorkerEvent>,
}

fn process(
    block_id: BlockId,
    block: Block,
    transaction_payload_worker: &mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
    milestone_payload_worker: &mpsc::UnboundedSender<MilestonePayloadWorkerEvent>,
    tagged_data_payload_worker: &mpsc::UnboundedSender<TaggedDataPayloadWorkerEvent>,
) {
    match block.payload() {
        Some(Payload::Transaction(_)) => {
            if transaction_payload_worker
                .send(TransactionPayloadWorkerEvent { block_id, block })
                .is_err()
            {
                error!("Sending block {} to transaction payload worker failed.", block_id);
            }
        }
        Some(Payload::Milestone(_)) => {
            if milestone_payload_worker
                .send(MilestonePayloadWorkerEvent { block_id, block })
                .is_err()
            {
                error!("Sending block {} to milestone payload worker failed.", block_id);
            }
        }
        Some(Payload::TaggedData(_)) => {
            if tagged_data_payload_worker
                .send(TaggedDataPayloadWorkerEvent {})
                .is_err()
            {
                error!("Sending block {} to tagged data payload worker failed.", block_id);
            }
        }
        _ => {}
    }
}

#[async_trait]
impl<N> Worker<N> for PayloadWorker
where
    N: Node,
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TransactionPayloadWorker>(),
            TypeId::of::<MilestonePayloadWorker>(),
            TypeId::of::<TaggedDataPayloadWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let transaction_payload_worker = node.worker::<TransactionPayloadWorker>().unwrap().tx.clone();
        let milestone_payload_worker = node.worker::<MilestonePayloadWorker>().unwrap().tx.clone();
        let tagged_data_payload_worker = node.worker::<TaggedDataPayloadWorker>().unwrap().tx.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(PayloadWorkerEvent { block_id, block }) = receiver.next().await {
                process(
                    block_id,
                    block,
                    &transaction_payload_worker,
                    &milestone_payload_worker,
                    &tagged_data_payload_worker,
                );
            }

            // Before the worker completely stops, the receiver needs to be drained for payloads to be analysed.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(PayloadWorkerEvent { block_id, block })) = receiver.next().now_or_never() {
                process(
                    block_id,
                    block,
                    &transaction_payload_worker,
                    &milestone_payload_worker,
                    &tagged_data_payload_worker,
                );
                count += 1;
            }

            debug!("Drained {} blocks.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
