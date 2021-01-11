// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{storage::StorageBackend, worker::TangleWorker};

use bee_message::{
    payload::{indexation::HashedIndex, Payload},
    MessageId,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_storage::access::Insert;
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, info, warn};
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) struct IndexationPayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct IndexationPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
}

async fn process<B: StorageBackend>(tangle: &MsTangle<B>, storage: &B, message_id: MessageId) {
    if let Some(message) = tangle.get(&message_id).await {
        let indexation = match message.payload() {
            Some(Payload::Indexation(indexation)) => indexation,
            Some(Payload::Transaction(transaction)) => match transaction.essence().payload() {
                Some(Payload::Indexation(indexation)) => indexation,
                _ => return,
            },
            _ => return,
        };
        let hash = indexation.hash();

        if let Err(e) = Insert::<(HashedIndex, MessageId), ()>::insert(&*storage, &(hash, message_id), &()).await {
            warn!("Inserting indexation payload failed: {:?}.", e);
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
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(IndexationPayloadWorkerEvent(message_id)) = receiver.next().await {
                process(&tangle, &storage, message_id).await;
            }

            let (_, mut receiver) = receiver.split();
            let receiver = receiver.get_mut();
            let mut count = 0;

            while let Ok(IndexationPayloadWorkerEvent(message_id)) = receiver.try_recv() {
                process(&tangle, &storage, message_id).await;
                count += 1;
            }

            debug!("Drained {} message ids.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
