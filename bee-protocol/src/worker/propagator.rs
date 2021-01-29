// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::MessageSolidified,
    storage::StorageBackend,
    worker::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent},
};

use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{metadata::IndexId, MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::*;
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
        // Skip messages that are already solid.
        if tangle.is_solid_message(message_id).await {
            continue;
        }

        // TODO Copying parents to avoid double locking, will be refactored.
        if let Some((parent1, parent2)) = tangle
            .get(&message_id)
            .await
            .map(|message| (*message.parent1(), *message.parent2()))
        {
            // If either of the parents is not yet solid, we do not propagate their current state.
            if !tangle.is_solid_message(&parent1).await || !tangle.is_solid_message(&parent2).await {
                continue;
            }

            // Note: There are two types of solidity carriers:
            // * solid messages (with available history)
            // * solid entry points / sep (with verified history)
            // Because we know both parents are solid, we also know that they have set OTRSI/YTRSI values, hence
            // we can simply unwrap. We also try to minimise unnecessary Tangle API calls if - for example - the
            // parent in question turns out to be a SEP.

            // Determine OTRSI/YTRSI of parent1
            let (parent1_otrsi, parent1_ytrsi) = match tangle.get_solid_entry_point_index(&parent1).await {
                Some(parent1_sepi) => (IndexId(parent1_sepi, parent1), IndexId(parent1_sepi, parent1)),
                None => match tangle.get_metadata(&parent1).await {
                    Some(parent1_md) => (parent1_md.otrsi().unwrap(), parent1_md.ytrsi().unwrap()),
                    None => continue,
                },
            };

            // Determine OTRSI/YTRSI of parent2
            let (parent2_otrsi, parent2_ytrsi) = match tangle.get_solid_entry_point_index(&parent2).await {
                Some(parent2_sepi) => (IndexId(parent2_sepi, parent2), IndexId(parent2_sepi, parent2)),
                None => match tangle.get_metadata(&parent2).await {
                    Some(parent2_md) => (parent2_md.otrsi().unwrap(), parent2_md.ytrsi().unwrap()),
                    None => continue,
                },
            };

            // Derive child OTRSI/YTRSI from its parents.
            // Note: The child inherits oldest (lowest) otrsi and the newest (highest) ytrsi from its parents.
            let child_otrsi = min(parent1_otrsi, parent2_otrsi);
            let child_ytrsi = max(parent1_ytrsi, parent2_ytrsi);

            let ms_index_maybe = tangle
                .update_metadata(&message_id, |metadata| {
                    // The child inherits the solid property from its parents.
                    metadata.solidify();

                    if metadata.flags().is_milestone() {
                        metadata.milestone_index()
                    } else {
                        metadata.set_otrsi(child_otrsi);
                        metadata.set_ytrsi(child_ytrsi);
                        None
                    }
                })
                .await
                .expect("Failed to fetch metadata.");

            // Try to propagate as far as possible into the future.
            if let Some(msg_children) = tangle.get_children(&message_id).await {
                for child in msg_children {
                    children.push(child);
                }
            }

            // Send child to the tip pool.
            if let Err(e) = tip_tx.send((*message_id, parent1, parent2, ms_index_maybe)).await {
                warn!("Failed to send message to the tip pool. Cause: {:?}", e);
            }
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
