// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    protocol::Protocol,
    tangle::MsTangle,
    worker::{MessageRequesterWorker, MessageRequesterWorkerEvent, RequestedMessages, TangleWorker},
};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_storage::storage::Backend;
use bee_tangle::traversal;

use async_trait::async_trait;
use futures::{channel::oneshot, StreamExt};
use log::{debug, info};

use std::{any::TypeId, convert::Infallible};

pub(crate) struct MilestoneSolidifierWorkerEvent(pub MilestoneIndex);

pub(crate) struct MilestoneSolidifierWorker {
    pub(crate) tx: flume::Sender<MilestoneSolidifierWorkerEvent>,
}

async fn trigger_solidification_unchecked<B: Backend>(
    tangle: &MsTangle<B>,
    message_requester: &flume::Sender<MessageRequesterWorkerEvent>,
    requested_messages: &RequestedMessages,
    target_index: MilestoneIndex,
    next_index: &mut MilestoneIndex,
) {
    if let Some(target_id) = tangle.get_milestone_message_id(target_index) {
        if !tangle.is_solid_message(&target_id) {
            debug!("Triggering solidification for milestone {}.", *target_index);

            // TODO: This wouldn't be necessary if the traversal code wasn't closure-driven
            let mut missing = Vec::new();

            traversal::visit_parents_depth_first(
                &**tangle,
                target_id,
                |id, _, metadata| {
                    (!metadata.flags().is_requested() || *id == target_id)
                        && !metadata.flags().is_solid()
                        && !requested_messages.contains_key(&id)
                },
                |_, _, _| {},
                |_, _, _| {},
                |missing_id| missing.push(*missing_id),
            );

            for missing_id in missing {
                Protocol::request_message(tangle, message_requester, requested_messages, missing_id, target_index)
                    .await;
            }
        }
        *next_index = target_index + MilestoneIndex(1);
    }
}

fn save_index(target_index: MilestoneIndex, queue: &mut Vec<MilestoneIndex>) {
    debug!("Storing milestone {}.", *target_index);
    if let Err(pos) = queue.binary_search_by(|index| target_index.cmp(index)) {
        queue.insert(pos, target_index);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneSolidifierWorker {
    type Config = oneshot::Receiver<MilestoneIndex>;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MessageRequesterWorker>(), TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut queue = vec![];
            let mut next_index = if let Ok(idx) = config.await {
                idx
            } else {
                return;
            };

            while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
                save_index(index, &mut queue);
                while let Some(index) = queue.pop() {
                    if index == next_index {
                        trigger_solidification_unchecked(
                            &tangle,
                            &message_requester,
                            &*requested_messages,
                            index,
                            &mut next_index,
                        )
                        .await;
                    } else {
                        queue.push(index);
                        break;
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
