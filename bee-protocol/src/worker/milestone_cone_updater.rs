// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MsTangle, worker::TangleWorker, Milestone, MilestoneIndex};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;

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
impl<N: Node> Worker<N> for MilestoneConeUpdaterWorker {
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

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(MilestoneConeUpdaterWorkerEvent(index, milestone)) = receiver.next().await {
                // When a new milestone gets solid, OTRSI and YTRSI of all messages that belong to the given cone
                // must be updated. Furthermore, updated values will be propagated to the future.
                update_messages_referenced_by_milestone::<N>(&tangle, milestone.message_id, index).await;
                // Update tip pool after all values got updated.
                tangle.update_tip_scores().await;
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

async fn update_messages_referenced_by_milestone<N: Node>(
    tangle: &MsTangle<N::Backend>,
    message_id: MessageId,
    milestone_index: MilestoneIndex,
) {
    let mut to_visit = vec![message_id];
    let mut visited = HashSet::new();

    while let Some(ref hash) = to_visit.pop() {
        if visited.contains(hash) {
            continue;
        } else {
            visited.insert(*hash);
        }

        if tangle.is_solid_entry_point(&hash) {
            continue;
        }

        // maybe the check below is not necessary; all messages from the most recent cone should be present
        if let Some(message) = tangle.get(&hash).await {
            // unwrap() is safe since message is present and so is the metadata
            if tangle.get_metadata(&hash).unwrap().cone_index().is_some() {
                continue;
            }

            tangle.update_metadata(&hash, |metadata| {
                metadata.set_cone_index(milestone_index);
                metadata.set_otrsi(milestone_index);
                metadata.set_ytrsi(milestone_index);
            });

            for child in tangle.get_children(&hash) {
                update_future_cone::<N>(tangle, child).await;
            }

            to_visit.push(*message.parent1());
            to_visit.push(*message.parent2());
        }
    }
}

async fn update_future_cone<N: Node>(tangle: &MsTangle<N::Backend>, child: MessageId) {
    let mut children = vec![child];
    while let Some(hash) = children.pop() {
        // maybe the check below is not necessary; all children from the most recent cone should be present
        if let Some(message) = tangle.get(&hash).await {
            // skip in case the message already got processed by update_messages_referenced_by_milestone()
            // unwrap() is safe since message is present and so is the metadata
            if tangle.get_metadata(&hash).unwrap().cone_index().is_some() {
                continue;
            }

            // get best OTRSI/YTRSI values from parents
            let parent1_otsri = tangle.otrsi(message.parent1());
            let parent2_otsri = tangle.otrsi(message.parent2());
            let parent1_ytrsi = tangle.ytrsi(message.parent1());
            let parent2_ytrsi = tangle.ytrsi(message.parent2());

            if parent1_otsri.is_none() || parent2_otsri.is_none() || parent1_ytrsi.is_none() || parent2_ytrsi.is_none()
            {
                continue;
            }

            // unwrap() is safe since None values are filtered above
            let best_otrsi = max(parent1_otsri.unwrap(), parent2_otsri.unwrap());
            let best_ytrsi = min(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

            // in case the messages already inherited the best OTRSI/YTRSI values, continue
            let current_otrsi = tangle.otrsi(&hash);
            let current_ytrsi = tangle.ytrsi(&hash);

            if let (Some(otrsi), Some(ytrsi)) = (current_otrsi, current_ytrsi) {
                if otrsi == best_otrsi && ytrsi == best_ytrsi {
                    continue;
                }
            }

            // update outdated OTRSI/YTRSI values
            tangle.update_metadata(&hash, |metadata| {
                metadata.set_otrsi(best_otrsi);
                metadata.set_ytrsi(best_ytrsi);
            });

            // propagate to children
            for child in tangle.get_children(&hash) {
                children.push(child);
            }
        }
    }
}
