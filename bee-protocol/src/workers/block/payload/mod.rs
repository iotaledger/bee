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
    pub(crate) message_id: BlockId,
    pub(crate) message: Block,
}

pub(crate) struct PayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<PayloadWorkerEvent>,
}

fn process(
    message_id: BlockId,
    message: Block,
    transaction_payload_worker: &mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
    milestone_payload_worker: &mpsc::UnboundedSender<MilestonePayloadWorkerEvent>,
    tagged_data_payload_worker: &mpsc::UnboundedSender<TaggedDataPayloadWorkerEvent>,
) {
    match message.payload() {
        Some(Payload::Transaction(_)) => {
            if transaction_payload_worker
                .send(TransactionPayloadWorkerEvent { message_id, message })
                .is_err()
            {
                error!("Sending message {} to transaction payload worker failed.", message_id);
            }
        }
        Some(Payload::Milestone(_)) => {
            if milestone_payload_worker
                .send(MilestonePayloadWorkerEvent { message_id, message })
                .is_err()
            {
                error!("Sending message {} to milestone payload worker failed.", message_id);
            }
        }
        Some(Payload::TaggedData(_)) => {
            if tagged_data_payload_worker
                .send(TaggedDataPayloadWorkerEvent {})
                .is_err()
            {
                error!("Sending message {} to tagged data payload worker failed.", message_id);
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

            while let Some(PayloadWorkerEvent { message_id, message }) = receiver.next().await {
                process(
                    message_id,
                    message,
                    &transaction_payload_worker,
                    &milestone_payload_worker,
                    &tagged_data_payload_worker,
                );
            }

            // Before the worker completely stops, the receiver needs to be drained for payloads to be analysed.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(PayloadWorkerEvent { message_id, message })) = receiver.next().now_or_never() {
                process(
                    message_id,
                    message,
                    &transaction_payload_worker,
                    &milestone_payload_worker,
                    &tagged_data_payload_worker,
                );
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
