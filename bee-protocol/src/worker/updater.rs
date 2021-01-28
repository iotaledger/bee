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
pub(crate) struct IndexUpdaterWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Milestone);

pub(crate) struct IndexUpdaterWorker {
    pub(crate) tx: mpsc::UnboundedSender<IndexUpdaterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for IndexUpdaterWorker
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
            info!("xTRSI updater started.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(IndexUpdaterWorkerEvent(index, milestone)) = receiver.next().await {
                process(&tangle, milestone, index).await;
            }

            // Before the worker completely stops, the receiver needs to be drained for milestone cones to be updated.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(IndexUpdaterWorkerEvent(index, milestone))) = receiver.next().now_or_never() {
                process(&tangle, milestone, index).await;
                count += 1;
            }

            debug!("Drained {} milestones.", count);

            info!("xTRSI updater stopped.");
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
        // Update the past cone of this milestone by setting its milestone index, and return them.
        let roots = update_past_cone(tangle, parent1, parent2, index).await;

        // Propagate new otrsi/ytrsi
        update_future_cone(tangle, roots).await;

        // Update tip pool after all values got updated.
        tangle.update_tip_scores().await;
    }
}

async fn update_past_cone<B: StorageBackend>(
    tangle: &MsTangle<B>,
    parent1: MessageId,
    parent2: MessageId,
    index: MilestoneIndex,
) -> HashSet<MessageId> {
    let mut parents = vec![parent1, parent2];
    let mut updated = HashSet::new();

    while let Some(parent_id) = parents.pop() {
        // Our stop conditions. Note that the order of calls is important (from cheap to more expensive) for performance
        // reasons.
        if updated.contains(&parent_id)
            || tangle.is_solid_entry_point(&parent_id).await
            || tangle
                .get_metadata(&parent_id)
                .await
                .unwrap()
                .milestone_index()
                .is_some()
        {
            continue;
        }

        tangle
            .update_metadata(&parent_id, |metadata| {
                // TODO: Throw one of those indexes away ;)
                metadata.set_milestone_index(index);
                metadata.set_otrsi(IndexId(index, parent_id));
                metadata.set_ytrsi(IndexId(index, parent_id));
            })
            .await;

        if let Some((grand_parent1, grand_parent2)) = tangle
            .get(&parent_id)
            .await
            .map(|parent| (*parent.parent1(), *parent.parent2()))
        {
            parents.push(grand_parent1);
            parents.push(grand_parent2);
        }

        // Preferably we would only collect the 'root messages/transactions'. They are defined as being confirmed by
        // a milestone, but at least one of their children is not confirmed yet. One can think of them as an attachment
        // point for new messages to the main tangle. It is ensured however, that this set *contains* the root messages
        // as well, and during the future walk we will skip already confirmed children, which shouldn't be a performance
        // issue.
        updated.insert(parent_id);
    }

    debug!("Updated milestone index for {} messages.", updated.len());

    updated
}

// NOTE: so once a milestone comes in we have to walk the future cones of the root transactions and update their
// OTRSI and YTRSI; during that time we need to block the propagator, otherwise it will propagate outdated data.
async fn update_future_cone<B: StorageBackend>(tangle: &MsTangle<B>, roots: HashSet<MessageId>) {
    let mut to_process = roots.into_iter().collect::<Vec<_>>();
    let mut processed = Vec::new();

    while let Some(parent_id) = to_process.pop() {
        if let Some(children) = tangle.get_children(&parent_id).await {
            // Unwrap is safe with very high probability.
            let (parent_otrsi, parent_ytrsi) = tangle
                .get_metadata(&parent_id)
                .await
                .map(|md| (md.otrsi(), md.ytrsi()))
                .unwrap();

            // TODO: investigate data race
            // Skip vertices with unset otrsi/ytrsi
            let (parent_otrsi, parent_ytrsi) = {
                if parent_otrsi.is_none() || parent_ytrsi.is_none() {
                    continue;
                } else {
                    (parent_otrsi.unwrap(), parent_ytrsi.unwrap())
                }
            };

            // We can update the OTRSI/YTRSI of those children that inherited the value from the current parent.
            for child in &children {
                if let Some(child_metadata) = tangle.get_metadata(&child).await {
                    // We can ignore children that are already confirmed
                    // TODO: resolve ambiguity betwenn `is_confirmed()` and `milestone_index().is_some()`
                    // if child_metadata.flags().is_confirmed() {
                    if child_metadata.milestone_index().is_some() {
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

    debug!("Updated xTRSI values for {} messages.", processed.len());
}
