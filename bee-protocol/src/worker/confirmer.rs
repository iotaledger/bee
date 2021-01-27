// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::StorageBackend;

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    MessageId,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{metadata::IndexId, MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, collections::HashSet, convert::Infallible};

#[derive(Debug)]
pub(crate) struct ConfirmationWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Milestone);

pub(crate) struct ConfirmationWorker {
    pub(crate) tx: mpsc::UnboundedSender<ConfirmationWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for ConfirmationWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(ConfirmationWorkerEvent(index, milestone)) = receiver.next().await {
                process(&tangle, milestone, index).await;
            }

            // Before the worker completely stops, the receiver needs to be drained for milestone cones to be updated.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(ConfirmationWorkerEvent(index, milestone))) = receiver.next().now_or_never() {
                process(&tangle, milestone, index).await;
                count += 1;
            }

            debug!("Drained {} milestones.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

async fn process<B: StorageBackend>(tangle: &MsTangle<B>, milestone: Milestone, index: MilestoneIndex) {
    let message_id = milestone.message_id();

    if let Some((parent1, parent2)) = tangle
        .get(message_id)
        .await
        .map(|message| (*message.parent1(), *message.parent2()))
    {
        // Confirm the past cone of this milestone, and return all newly confir
        let confirmed = confirm_past_cone(tangle, parent1, parent2, index).await;

        // Propagate new confirmation states
        update_otrsi_ytrsi(tangle, confirmed).await;
    }

    // Update tip pool after all values got updated.
    tangle.update_tip_scores().await;
}

async fn confirm_past_cone<B: StorageBackend>(
    tangle: &MsTangle<B>,
    parent1: MessageId,
    parent2: MessageId,
    index: MilestoneIndex,
) -> HashSet<MessageId> {
    let mut parents = vec![parent1, parent2];
    let mut confirmed = HashSet::new();

    while let Some(id) = parents.pop() {
        // Our stop conditions. Note that the order of calls is important (from cheap to more expensive) for performance
        // reasons.
        if confirmed.contains(&id)
            || tangle.is_solid_entry_point(&id)
            || tangle.get_metadata(&id).await.unwrap().flags().is_confirmed()
        {
            continue;
        }

        tangle
            .update_metadata(&id, |metadata| {
                // TODO: Throw one of those indexes away ;)
                metadata.set_milestone_index(index);
                metadata.set_cone_index(index);
                metadata.set_otrsi(IndexId(index, id));
                metadata.set_ytrsi(IndexId(index, id));
                metadata.flags_mut().set_confirmed(true);
            })
            .await;

        if let Some((parent1, parent2)) = tangle
            .get(&id)
            .await
            .map(|message| (*message.parent1(), *message.parent2()))
        {
            parents.push(parent1);
            parents.push(parent2);
        }

        // Preferably we would only collect the 'root messages/transactions'. They are defined as being confirmed by
        // a milestone, but at least one of their children is not confirmed yet. One can think of them as an attachment
        // point for new messages to the main tangle. It is ensured however, that this set *contains* the root messages
        // as well, and during the future walk we will skip already confirmed children, which shouldn't be a performance
        // issue.
        confirmed.insert(id);
    }

    debug!("Confirmed {} messages.", confirmed.len());

    confirmed
}

// NOTE: so once a milestone comes in we have to walk the future cones of the root transactions and update their
// OTRSI and YTRSI; during that time we need to block the propagator, otherwise it will propagate outdated data.
async fn update_otrsi_ytrsi<B: StorageBackend>(tangle: &MsTangle<B>, confirmed: HashSet<MessageId>) {
    let mut to_process = confirmed.into_iter().collect::<Vec<_>>();
    let mut processed = Vec::new();

    while let Some(parent_id) = to_process.pop() {
        if let Some(children) = tangle.get_children(&parent_id).await {
            // Unwrap is safe with very high probability.
            let (parent_otrsi, parent_ytrsi) = tangle
                .get_metadata(&parent_id)
                .await
                .map(|md| (md.otrsi().unwrap(), md.ytrsi().unwrap()))
                .unwrap();

            // We can update the OTRSI/YTRSI of those children that inherited the value from the current parent.
            for child in &children {
                if let Some(child_metadata) = tangle.get_metadata(&child).await {
                    // We can ignore children that are already confirmed
                    if child_metadata.flags().is_confirmed() {
                        continue;
                    }

                    // If the childs OTRSI was previously inherited from the current parent, update it.
                    if let Some(child_otrsi) = child_metadata.otrsi() {
                        if child_otrsi.1.eq(&parent_id) {
                            tangle
                                .update_metadata(child, |md| {
                                    md.set_otrsi(IndexId(parent_otrsi.0, parent_id));
                                })
                                .await;
                        }
                    }

                    // If the childs YTRSI was previously inherited from the current parent, update it.
                    if let Some(child_ytrsi) = child_metadata.ytrsi() {
                        if child_ytrsi.1.eq(&parent_id) {
                            tangle
                                .update_metadata(child, |md| {
                                    md.set_ytrsi(IndexId(parent_ytrsi.0, parent_id));
                                })
                                .await;
                        }
                    }

                    // Continue the future walk for that child, if we haven't landed on it earlier already.
                    if !processed.contains(child) {
                        to_process.push(*child);
                    }
                }
            }

            processed.push(parent_id);
        }
    }

    debug!("Finished updating OTRSI/YTRSI values");
}
