// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MsTangle, worker::TangleWorker};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct MessageValidatorWorkerEvent(pub(crate) MessageId);

pub(crate) struct MessageValidatorWorker {
    pub(crate) tx: flume::Sender<MessageValidatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageValidatorWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MessageValidatorWorkerEvent(id)) = receiver.next().await {
                if let Some(message) = tangle.get(&id).await {
                    tangle.insert_tip(id, *message.parent1(), *message.parent2()).await;
                    // if let Ok(bundle) = builder.validate() {
                    //     tangle.update_metadata(&hash, |metadata| {
                    //         metadata.flags_mut().set_valid(true);
                    //     });
                    // }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
