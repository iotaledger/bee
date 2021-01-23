// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::MessageSolidified,
    storage::StorageBackend,
    worker::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, TangleWorker},
};

use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{
    any::TypeId,
    cmp::{max, min},
    convert::Infallible,
};

#[derive(Debug)]
pub(crate) struct PropagatorWorkerEvent(pub(crate) MessageId);

pub(crate) struct PropagatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<PropagatorWorkerEvent>,
}

async fn propagate<B: StorageBackend>(
    message_id: MessageId,
    tangle: &MsTangle<B>,
    tip_tx: &async_channel::Sender<(MessageId, MessageId, MessageId, Option<MilestoneIndex>)>,
) {
    let mut children = vec![message_id];

    while let Some(ref message_id) = children.pop() {
        if tangle.is_solid_message(message_id).await {
            continue;
        }

        // TODO Copying parents to avoid double locking, will be refactored.
        if let Some((parent1, parent2)) = tangle
            .get(&message_id)
            .await
            .map(|message| (*message.parent1(), *message.parent2()))
        {
            if !tangle.is_solid_message(&parent1).await || !tangle.is_solid_message(&parent2).await {
                continue;
            }

            // get OTRSI/YTRSI from parents
            let parent1_otsri = tangle.otrsi(&parent1).await;
            let parent2_otsri = tangle.otrsi(&parent2).await;
            let parent1_ytrsi = tangle.ytrsi(&parent1).await;
            let parent2_ytrsi = tangle.ytrsi(&parent2).await;

            // get best OTRSI/YTRSI from parents
            // unwrap() is safe because parents are solid which implies that OTRSI/YTRSI values are
            // available.
            let best_otrsi = max(parent1_otsri.unwrap(), parent2_otsri.unwrap());
            let best_ytrsi = min(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

            tangle
                .update_metadata(&message_id, |metadata| {
                    // OTRSI/YTRSI values need to be set before the solid flag, to ensure that the
                    // MilestoneConeUpdater is aware of all values.
                    metadata.set_otrsi(best_otrsi);
                    metadata.set_ytrsi(best_ytrsi);
                    metadata.solidify();
                })
                .await;

            let index = match tangle.get_metadata(&message_id).await {
                Some(meta) => {
                    if meta.flags().is_milestone() {
                        Some(meta.milestone_index())
                    } else {
                        None
                    }
                }
                None => None,
            };

            if let Some(msg_children) = tangle.get_children(&message_id).await {
                for child in msg_children {
                    children.push(child);
                }
            }

            let _ = tip_tx.send((*message_id, parent1, parent2, index)).await;
        }
    }
}

#[async_trait]
impl<N: Node> Worker<N> for PropagatorWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<MilestoneSolidifierWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let (tip_tx, tip_rx) = async_channel::unbounded();

            tokio::task::spawn({
                let tangle = tangle.clone();

                async move {
                    while let Ok((message_id, parent1, parent2, index)) = tip_rx.recv().await {
                        bus.dispatch(MessageSolidified(message_id));
                        tangle.insert_tip(message_id, parent1, parent2).await;

                        if let Some(index) = index {
                            if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(index)) {
                                error!("Sending solidification event failed: {}.", e);
                            }
                        }
                    }
                }
            });

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(PropagatorWorkerEvent(message_id)) = receiver.next().await {
                propagate(message_id, &tangle, &tip_tx).await;
            }

            // Before the worker completely stops, the receiver needs to be drained for statuses to be propagated.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(PropagatorWorkerEvent(message_id))) = receiver.next().now_or_never() {
                propagate(message_id, &tangle, &tip_tx).await;
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
