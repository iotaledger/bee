// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;

use std::convert::Infallible;

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

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(IndexationPayloadWorkerEvent(_message_id)) = receiver.next().await {
                // TODO store
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
