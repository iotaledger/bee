// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{packets::MessagePacket, storage::StorageBackend, HasherWorker, HasherWorkerEvent},
};

use bee_message::MessageId;
use bee_runtime::{
    node::Node,
    shutdown_stream::ShutdownStream,
    worker::{Error as WorkerError, Worker},
};

use async_trait::async_trait;
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info, trace};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, fmt};

pub(crate) fn notify_invalid_message(
    error: String,
    metrics: &NodeMetrics,
    notifier: Option<Sender<Result<MessageId, MessageSubmitterError>>>,
) {
    trace!("{}", error);
    metrics.invalid_messages_inc();

    if let Some(notifier) = notifier {
        if let Err(e) = notifier.send(Err(MessageSubmitterError(error))) {
            error!("Failed to send error: {:?}.", e);
        }
    }
}

pub(crate) fn notify_message(
    message_id: MessageId,
    notifier: Option<Sender<Result<MessageId, MessageSubmitterError>>>,
) {
    if let Some(notifier) = notifier {
        if let Err(e) = notifier.send(Ok(message_id)) {
            error!("Failed to send message id: {:?}.", e);
        }
    }
}

#[derive(Debug)]
pub struct MessageSubmitterError(pub String);

impl fmt::Display for MessageSubmitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct MessageSubmitterWorkerEvent {
    pub message: Vec<u8>,
    pub notifier: Sender<Result<MessageId, MessageSubmitterError>>,
}

pub struct MessageSubmitterWorker {
    pub tx: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageSubmitterWorker
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

            while let Some(MessageSubmitterWorkerEvent { message, notifier }) = receiver.next().await {
                let event = HasherWorkerEvent {
                    from: None,
                    message_packet: MessagePacket::new(message),
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
