// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::StorageBackend,
    worker::{IndexationPayloadWorker, IndexationPayloadWorkerEvent, TangleWorker},
};

use bee_message::{payload::Payload, MessageId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, info, warn};
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) struct TransactionPayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct TransactionPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
}

async fn process<B: StorageBackend>(
    tangle: &MsTangle<B>,
    message_id: MessageId,
    indexation_payload_worker: &mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
) {
    if let Some(message) = tangle.get(&message_id).await.map(|m| (*m).clone()) {
        if let Some(Payload::Transaction(transaction)) = message.payload() {
            if let Some(Payload::Indexation(_)) = transaction.essence().payload() {
                if let Err(e) = indexation_payload_worker.send(IndexationPayloadWorkerEvent(message_id)) {
                    warn!(
                        "Sending message id {} to indexation payload worker failed: {:?}.",
                        message_id, e
                    );
                }
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
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<IndexationPayloadWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let indexation_payload_worker = node.worker::<IndexationPayloadWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(TransactionPayloadWorkerEvent(message_id)) = receiver.next().await {
                process(&tangle, message_id, &indexation_payload_worker).await;
            }

            let (_, mut receiver) = receiver.split();
            let receiver = receiver.get_mut();
            let mut count = 0;

            while let Ok(TransactionPayloadWorkerEvent(message_id)) = receiver.try_recv() {
                process(&tangle, message_id, &indexation_payload_worker).await;
                count += 1;
            }

            debug!("Drained {} message ids.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
