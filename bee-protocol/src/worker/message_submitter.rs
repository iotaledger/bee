// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::Message,
    worker::{HasherWorker, HasherWorkerEvent},
};

use bee_common::{
    node::Node,
    shutdown_stream::ShutdownStream,
    worker::{Error as WorkerError, Worker},
};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info};

use std::{any::TypeId, fmt};

#[derive(Debug)]
pub struct MessageSubmitterError {
    pub reason: String,
}

impl fmt::Display for MessageSubmitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

pub struct MessageSubmitterWorkerEvent {
    pub buf: Vec<u8>,
    pub notifier: Sender<Result<MessageId, MessageSubmitterError>>,
}

pub struct MessageSubmitterWorker {
    pub tx: flume::Sender<MessageSubmitterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageSubmitterWorker {
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<HasherWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let hasher = node.worker::<HasherWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(event) = receiver.next().await {
                let event: MessageSubmitterWorkerEvent = event;
                let event = HasherWorkerEvent {
                    from: None,
                    message_packet: Message::new(&event.buf),
                    notifier: Some(event.notifier),
                };

                if let Err(e) = hasher.send(event) {
                    error!("Sending HasherWorkerEvent failed: {}", e);
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
