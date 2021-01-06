// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::StorageBackend,
    tangle::MsTangle,
    worker::{IndexationPayloadWorker, IndexationPayloadWorkerEvent, TangleWorker},
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::{payload::Payload, MessageId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{info, warn};
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) struct TransactionPayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct TransactionPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<TransactionPayloadWorkerEvent>,
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
                if let Some(message) = tangle.get(&message_id).await {
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

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
