// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::StorageBackend,
    worker::{MetricsWorker, TangleWorker},
    ProtocolMetrics,
};

use bee_message::{
    payload::{indexation::HashedIndex, Payload},
    MessageId,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_storage::access::Insert;
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) struct IndexationPayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct IndexationPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
}

async fn process<B: StorageBackend>(
    tangle: &MsTangle<B>,
    storage: &B,
    metrics: &ProtocolMetrics,
    message_id: MessageId,
) {
    if let Some(message) = tangle.get(&message_id).await.map(|m| (*m).clone()) {
        let indexation = match message.payload() {
            Some(Payload::Indexation(indexation)) => indexation,
            Some(Payload::Transaction(transaction)) => match transaction.essence().payload() {
                Some(Payload::Indexation(indexation)) => indexation,
                _ => return,
            },
            _ => return,
        };

        metrics.indexation_payload_inc(1);

        let hash = indexation.hash();

        if let Err(e) = Insert::<(HashedIndex, MessageId), ()>::insert(&*storage, &(hash, message_id), &()).await {
            error!("Inserting indexation payload failed: {:?}.", e);
        }
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
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(IndexationPayloadWorkerEvent(message_id)) = receiver.next().await {
                process(&tangle, &storage, &metrics, message_id).await;
            }

            // Before the worker completely stops, the receiver needs to be drained for indexation payloads to be
            // analysed. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(IndexationPayloadWorkerEvent(message_id))) = receiver.next().now_or_never() {
                process(&tangle, &storage, &metrics, message_id).await;
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
