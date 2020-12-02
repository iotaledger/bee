// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::MessageSolidified,
    tangle::MsTangle,
    worker::{MessageValidatorWorker, MessageValidatorWorkerEvent, TangleWorker},
};

use bee_common::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{error, info, warn};

use std::{
    any::TypeId,
    cmp::{max, min},
    convert::Infallible,
};

pub(crate) struct PropagatorWorkerEvent {
    pub(crate) message_id: MessageId,
    pub(crate) propagator_notifier: futures::channel::oneshot::Sender<()>,
}

pub(crate) struct PropagatorWorker {
    pub(crate) tx: flume::Sender<PropagatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for PropagatorWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MessageValidatorWorker>(), TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let message_validator = node.worker::<MessageValidatorWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let bus = node.resource::<Bus>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(event) = receiver.next().await {
                let event: PropagatorWorkerEvent = event;

                let mut children = vec![event.message_id];

                while let Some(ref hash) = children.pop() {
                    if tangle.is_solid_message(hash) {
                        continue;
                    }

                    if let Some(message) = tangle.get(&hash).await {
                        if tangle.is_solid_message(message.parent1()) && tangle.is_solid_message(message.parent2()) {
                            // get otrsi and ytrsi from parents
                            let parent1_otsri = tangle.otrsi(message.parent1());
                            let parent2_otsri = tangle.otrsi(message.parent2());
                            let parent1_ytrsi = tangle.ytrsi(message.parent1());
                            let parent2_ytrsi = tangle.ytrsi(message.parent2());

                            let best_otrsi = max(parent1_otsri.unwrap(), parent2_otsri.unwrap());
                            let best_ytrsi = min(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

                            tangle.update_metadata(&hash, |metadata| {
                                metadata.solidify();
                                metadata.set_otrsi(best_otrsi);
                                metadata.set_ytrsi(best_ytrsi);
                            });

                            for child in tangle.get_children(&hash) {
                                children.push(child);
                            }

                            bus.dispatch(MessageSolidified(*hash));

                            if let Err(e) = message_validator.send(MessageValidatorWorkerEvent(*hash)) {
                                warn!("Failed to send hash to message validator: {:?}.", e);
                            }
                        }
                    }
                }

                if let Err(e) = event.propagator_notifier.send(()) {
                    error!("Failed to report back from propagator: {:?}.", e);
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
