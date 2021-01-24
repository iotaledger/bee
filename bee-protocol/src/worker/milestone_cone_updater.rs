// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{storage::StorageBackend, worker::TangleWorker};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    MessageId,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{
    any::TypeId,
    cmp::{max, min},
    collections::HashSet,
    convert::Infallible,
};

#[derive(Debug)]
pub(crate) struct MilestoneConeUpdaterWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Milestone);

pub(crate) struct MilestoneConeUpdaterWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneConeUpdaterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneConeUpdaterWorker
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

            while let Some(MilestoneConeUpdaterWorkerEvent(index, milestone)) = receiver.next().await {
                process(&tangle, index, milestone).await;
            }

            // Before the worker completely stops, the receiver needs to be drained for milestone cones to be updated.
            // Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(MilestoneConeUpdaterWorkerEvent(index, milestone))) = receiver.next().now_or_never() {
                process(&tangle, index, milestone).await;
                count += 1;
            }

            debug!("Drained {} milestones.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

async fn process<B: StorageBackend>(tangle: &MsTangle<B>, index: MilestoneIndex, milestone: Milestone) {
    // sets cone_index + updates OTRSI/YTRSI  for every message that belongs to the milestone cone
    let confirmed_messages = update_past_cone(tangle, *milestone.message_id(), index).await;
    // propagates the updated OTRSI/YTRSI values to the future cones
    update_future_cones(tangle, confirmed_messages.into_iter().collect()).await;
    // Update tip pool after all values got updated.
    tangle.update_tip_scores().await;
}

async fn update_past_cone<B: StorageBackend>(
    tangle: &MsTangle<B>,
    message_id: MessageId,
    milestone_index: MilestoneIndex,
) -> HashSet<MessageId> {
    let mut to_visit = vec![message_id];
    let mut visited = HashSet::new();

    while let Some(ref message_id) = to_visit.pop() {
        if visited.contains(message_id) {
            continue;
        } else {
            visited.insert(*message_id);
        }

        if tangle.is_solid_entry_point(&message_id) {
            continue;
        }

        // maybe the check below is not necessary; all messages from the most recent cone should be present
        if let Some((parent1, parent2)) = tangle.get(&message_id).await.map(|m| (*m.parent1(), *m.parent2())) {
            // unwrap() is safe since message is present and so is the metadata
            if tangle.get_metadata(&message_id).await.unwrap().cone_index().is_some() {
                continue;
            }

            tangle
                .update_metadata(&message_id, |metadata| {
                    metadata.set_cone_index(milestone_index);
                    metadata.set_otrsi(milestone_index);
                    metadata.set_ytrsi(milestone_index);
                })
                .await;

            to_visit.push(parent1);
            to_visit.push(parent2);
        }
    }

    visited
}

async fn update_future_cones<B: StorageBackend>(tangle: &MsTangle<B>, mut confirmed_messages: Vec<MessageId>) {
    let mut children_visited = HashSet::new();

    while let Some(confirmed_message) = confirmed_messages.pop() {
        if let Some(children) = tangle.get_children(&confirmed_message).await {
            let mut children_to_visit: Vec<MessageId> = children.into_iter().collect();

            while let Some(message_id) = children_to_visit.pop() {
                if children_visited.contains(&message_id) {
                    continue;
                } else {
                    children_visited.insert(message_id);
                }

                // maybe the check below is not necessary; all children from the most recent cone should be present
                if let Some((parent1, parent2)) = tangle.get(&message_id).await.map(|m| (*m.parent1(), *m.parent2())) {
                    // skip in case the message already got processed by update_past_cone()
                    // unwrap() is safe since message is present and so is the metadata
                    if tangle.get_metadata(&message_id).await.unwrap().cone_index().is_some() {
                        continue;
                    }

                    // get the OTRSI/YTRSI values from parents
                    let parent1_otsri = tangle.otrsi(&parent1).await;
                    let parent2_otsri = tangle.otrsi(&parent2).await;
                    let parent1_ytrsi = tangle.ytrsi(&parent1).await;
                    let parent2_ytrsi = tangle.ytrsi(&parent2).await;

                    if parent1_otsri.is_none()
                        || parent2_otsri.is_none()
                        || parent1_ytrsi.is_none()
                        || parent2_ytrsi.is_none()
                    {
                        continue;
                    }

                    // unwrap() is safe since None values are filtered above
                    let new_otrsi = min(parent1_otsri.unwrap(), parent2_otsri.unwrap());
                    let new_ytrsi = max(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

                    // in case the messages already inherited the best OTRSI/YTRSI values, continue
                    let current_otrsi = tangle.otrsi(&message_id).await;
                    let current_ytrsi = tangle.ytrsi(&message_id).await;

                    if let (Some(otrsi), Some(ytrsi)) = (current_otrsi, current_ytrsi) {
                        if otrsi == new_otrsi && ytrsi == new_ytrsi {
                            continue;
                        }
                    }

                    // update outdated OTRSI/YTRSI values
                    tangle
                        .update_metadata(&message_id, |metadata| {
                            metadata.set_otrsi(new_otrsi);
                            metadata.set_ytrsi(new_ytrsi);
                        })
                        .await;

                    // propagate to children
                    if let Some(msg_children) = tangle.get_children(&message_id).await {
                        for child in msg_children {
                            children_to_visit.push(child);
                        }
                    }
                }
            }
        }
    }
}
