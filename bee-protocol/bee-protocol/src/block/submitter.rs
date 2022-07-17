// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, fmt};

use async_trait::async_trait;
use bee_block::BlockId;
use bee_runtime::{
    node::Node,
    shutdown_stream::ShutdownStream,
    worker::{Error as WorkerError, Worker},
};
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info, trace};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    packets::BlockPacket, storage::StorageBackend, types::metrics::NodeMetrics, HasherWorker, HasherWorkerEvent,
};

pub(crate) fn notify_invalid_block(
    error: String,
    metrics: &NodeMetrics,
    notifier: Option<Sender<Result<BlockId, BlockSubmitterError>>>,
) {
    trace!("{}", error);
    metrics.invalid_blocks_inc();

    if let Some(notifier) = notifier {
        if let Err(e) = notifier.send(Err(BlockSubmitterError(error))) {
            error!("Failed to send error: {:?}.", e);
        }
    }
}

pub(crate) fn notify_block(block_id: BlockId, notifier: Option<Sender<Result<BlockId, BlockSubmitterError>>>) {
    if let Some(notifier) = notifier {
        if let Err(e) = notifier.send(Ok(block_id)) {
            error!("Failed to send block id: {:?}.", e);
        }
    }
}

#[derive(Debug)]
pub struct BlockSubmitterError(pub String);

impl fmt::Display for BlockSubmitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BlockSubmitterError {}

pub struct BlockSubmitterWorkerEvent {
    pub block: Vec<u8>,
    pub notifier: Sender<Result<BlockId, BlockSubmitterError>>,
}

pub struct BlockSubmitterWorker {
    pub tx: mpsc::UnboundedSender<BlockSubmitterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BlockSubmitterWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<HasherWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let hasher = node.worker::<HasherWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(BlockSubmitterWorkerEvent { block, notifier }) = receiver.next().await {
                let event = HasherWorkerEvent {
                    from: None,
                    block_packet: BlockPacket::new(block),
                    notifier: Some(notifier),
                };
                if let Err(e) = hasher.send(event) {
                    error!("Sending HasherWorkerEvent failed: {}.", e);
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
