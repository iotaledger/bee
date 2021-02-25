// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, model::Receipt, storage::StorageBackend};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct ReceiptValidatorWorkerEvent(pub Receipt);

pub struct ReceiptValidatorWorker {
    pub tx: mpsc::UnboundedSender<ReceiptValidatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for ReceiptValidatorWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Error;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(ReceiptValidatorWorkerEvent(_receipt)) = receiver.next().await {}

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
