// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::Message,
    storage::Backend,
    worker::{HasherWorker, HasherWorkerEvent},
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{
    node::Node,
    worker::{Error as WorkerError, Worker},
};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info};
use tokio::sync::mpsc;

use std::{any::TypeId, fmt};

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
    N::Backend: Backend,
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

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(MessageSubmitterWorkerEvent { message, notifier }) = receiver.next().await {
                let event = HasherWorkerEvent {
                    from: None,
                    message_packet: Message::new(&message),
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
