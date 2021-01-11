// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MsTangle, worker::TangleWorker};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;

use crate::event::TipAdded;
use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) struct MessageValidatorWorkerEvent(pub(crate) MessageId);

pub(crate) struct MessageValidatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<MessageValidatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageValidatorWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let bus = node.bus();
        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(MessageValidatorWorkerEvent(id)) = receiver.next().await {
                if let Some(message) = tangle.get(&id).await {
                    tangle.insert_tip(id, *message.parent1(), *message.parent2()).await;
                    bus.dispatch(TipAdded(id));
                    // TODO validate
                    //     tangle.update_metadata(&hash, |metadata| {
                    //         metadata.flags_mut().set_valid(true);
                    //     });
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
