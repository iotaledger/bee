// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, collections::HashSet, convert::Infallible};

use async_trait::async_trait;
use bee_message::{milestone::Milestone, payload::milestone::MilestoneIndex, MessageId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{message_metadata::IndexId, Tangle, TangleWorker};
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::workers::storage::StorageBackend;

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

        let tangle = node.resource::<Tangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

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

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

async fn process<B: StorageBackend>(tangle: &Tangle<B>, milestone: Milestone, index: MilestoneIndex) {
    if let Some(parents) = tangle
        .get(milestone.message_id())
        .map(|message| message.parents().to_vec())
    {
        // Update the past cone of this milestone by setting its milestone index, and return them.
        let roots = update_past_cone(tangle, parents, index).await;

        // Note: For tip-selection only the most recent tangle is relevent. That means that during synchronization we do
        // not need to update xMRSI values or tip scores before (LATEST_MILESTONE_INDEX - BELOW_MAX_DEPTH).
        if index > tangle.get_latest_milestone_index() - tangle.config().below_max_depth() {
            update_future_cone(tangle, roots);

            // Update tip pool after all values got updated.
            tangle.update_tip_scores().await;
        }
    }
}

async fn update_past_cone<B: StorageBackend>(
    tangle: &Tangle<B>,
    mut parents: Vec<MessageId>,
    index: MilestoneIndex,
) -> HashSet<MessageId> {
    let mut updated = HashSet::new();

    while let Some(parent_id) = parents.pop() {
        // Our skip conditions:
        // 1) check if we already updated it during this run
        // 2) check if it's a SEP
        // 3) check if we already updated it during a previous run
        // Note that the order of calls is important (from cheap to more expensive) for performance reasons.
        if updated.contains(&parent_id)
            || tangle.is_solid_entry_point(&parent_id).await
            || tangle
                .get_metadata(&parent_id)
                // TODO: I don't think unwrapping here is safe. Investigate!
                .unwrap()
                .milestone_index()
                .is_some()
        {
            continue;
        }

        tangle.update_metadata(&parent_id, |metadata| {
            metadata.set_milestone_index(index);

            let index = IndexId::new(index, parent_id);
            metadata.set_omrsi_and_ymrsi(index, index);
        });

        if let Some(parent) = tangle.get(&parent_id) {
            parents.extend_from_slice(parent.parents())
        }

        // Preferably we would only collect the 'root messages/transactions'. They are defined as being confirmed by
        // a milestone, but at least one of their children is not confirmed yet. One can think of them as an attachment
        // point for new messages to the main tangle. It is ensured however, that this set *contains* the root messages
        // as well, and during the future walk we will skip already confirmed children, which shouldn't be a performance
        // issue.
        updated.insert(parent_id);
    }

    debug!("Set milestone index {} to {} messages.", index, updated.len());

    updated
}

// NOTE: Once a milestone comes in we have to walk the future cones of the root transactions and update their OMRSI and
// YMRSI; during that time we need to block the propagator, otherwise it will propagate outdated data.
fn update_future_cone<B: StorageBackend>(tangle: &Tangle<B>, roots: HashSet<MessageId>) {
    let mut to_process = roots.into_iter().collect::<Vec<_>>();
    let mut processed = HashSet::new();

    while let Some(parent_id) = to_process.pop() {
        if let Some(children) = tangle.get_children(&parent_id) {
            // Unwrap is safe with very high probability.
            let parent_omrsi_and_ymrsi = tangle.get_metadata(&parent_id).map(|md| md.omrsi_and_ymrsi()).unwrap();

            // TODO: investigate data race
            // Skip vertices with unset omrsi/ymrsi
            match parent_omrsi_and_ymrsi {
                None => continue,
                Some((parent_omrsi, parent_ymrsi)) => {
                    // We can update the OMRSI/YMRSI of those children that inherited the value from the current parent.
                    for child in &children {
                        let continue_walk = tangle
                            .update_metadata(child, |child_metadata| {
                                // We can ignore children that are already confirmed
                                // TODO: resolve ambiguity between `is_confirmed()` and `milestone_index().is_some()`
                                // if child_metadata.flags().is_confirmed() {
                                if child_metadata.milestone_index().is_some() {
                                    return false;
                                }

                                // If the childs OMRSI and YMRSI was previously inherited from the current parent,
                                // update it.
                                child_metadata.update_omrsi_and_ymrsi(|child_omrsi, child_ymrsi| {
                                    if child_omrsi.id() == parent_id {
                                        *child_omrsi = IndexId::new(parent_omrsi.index(), parent_id);
                                    }

                                    if child_ymrsi.id() == parent_id {
                                        *child_ymrsi = IndexId::new(parent_ymrsi.index(), parent_id);
                                    }
                                });

                                true
                            })
                            .unwrap_or_default();

                        // Continue the future walk for that child, if we haven't landed on it earlier already.
                        if continue_walk && !processed.contains(child) {
                            to_process.push(*child);
                        }
                    }

                    processed.insert(parent_id);
                }
            }
        }
    }

    debug!("Updated xMRSI values for {} messages.", processed.len());
}
