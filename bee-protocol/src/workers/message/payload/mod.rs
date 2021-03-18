// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod indexation;
mod milestone;
mod transaction;

pub(crate) use indexation::{IndexationPayloadWorker, IndexationPayloadWorkerEvent};
pub(crate) use milestone::{MilestonePayloadWorker, MilestonePayloadWorkerEvent};
pub(crate) use transaction::{TransactionPayloadWorker, TransactionPayloadWorkerEvent};

use crate::storage::StorageBackend;

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
pub(crate) struct PayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct PayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<PayloadWorkerEvent>,
}

async fn process<B: StorageBackend>(
    tangle: &MsTangle<B>,
    message_id: MessageId,
    transaction_payload_worker: &mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
    milestone_payload_worker: &mpsc::UnboundedSender<MilestonePayloadWorkerEvent>,
    indexation_payload_worker: &mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
) {
    if let Some(message) = tangle.get(&message_id).await.map(|m| (*m).clone()) {
        match message.payload() {
            Some(Payload::Transaction(_)) => {
                if let Err(e) = transaction_payload_worker.send(TransactionPayloadWorkerEvent(message_id)) {
                    error!(
                        "Sending message id {} to transaction payload worker failed: {:?}.",
                        message_id, e
                    );
                }
            }
            Some(Payload::Milestone(_)) => {
                if let Err(e) = milestone_payload_worker.send(MilestonePayloadWorkerEvent(message_id)) {
                    error!(
                        "Sending message id {} to milestone payload worker failed: {:?}.",
                        message_id, e
                    );
                }
            }
            Some(Payload::Indexation(_)) => {
                if let Err(e) = indexation_payload_worker.send(IndexationPayloadWorkerEvent(message_id)) {
                    error!(
                        "Sending message id {} to indexation payload worker failed: {:?}.",
                        message_id, e
                    );
                }
            }
            _ => {}
        }
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
            TypeId::of::<TangleWorker>(),
            TypeId::of::<TransactionPayloadWorker>(),
            TypeId::of::<MilestonePayloadWorker>(),
            TypeId::of::<IndexationPayloadWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let transaction_payload_worker = node.worker::<TransactionPayloadWorker>().unwrap().tx.clone();
        let milestone_payload_worker = node.worker::<MilestonePayloadWorker>().unwrap().tx.clone();
        let indexation_payload_worker = node.worker::<IndexationPayloadWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(PayloadWorkerEvent(message_id)) = receiver.next().await {
                process(
                    &tangle,
                    message_id,
                    &transaction_payload_worker,
                    &milestone_payload_worker,
                    &indexation_payload_worker,
                )
                .await;
            }

            // Before the worker completely stops, the receiver needs to be drained for payloads to be analysed.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(PayloadWorkerEvent(message_id))) = receiver.next().now_or_never() {
                process(
                    &tangle,
                    message_id,
                    &transaction_payload_worker,
                    &milestone_payload_worker,
                    &indexation_payload_worker,
                )
                .await;
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
