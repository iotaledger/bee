// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MsTangle, worker::TangleWorker};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::{payload::Payload, MessageId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) struct IndexationPayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct IndexationPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<IndexationPayloadWorkerEvent>,
}

#[async_trait]
impl<N> Worker<N> for IndexationPayloadWorker
where
    N: Node,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(IndexationPayloadWorkerEvent(message_id)) = receiver.next().await {
                if let Some(message) = tangle.get(&message_id).await {
                    let indexation = match message.payload() {
                        Some(Payload::Indexation(indexation)) => indexation,
                        Some(Payload::Transaction(transaction)) => match transaction.essence().payload() {
                            Some(Payload::Indexation(indexation)) => indexation,
                            _ => continue,
                        },
                        _ => continue,
                    };
                    let _hash = indexation.hash();
                    // TODO store
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
