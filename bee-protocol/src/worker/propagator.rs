// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::{LatestSolidMilestoneChanged, MessageSolidified},
    storage::StorageBackend,
    worker::{
        milestone_cone_updater::{MilestoneConeUpdaterWorker, MilestoneConeUpdaterWorkerEvent},
        TangleWorker,
    },
};

use bee_message::MessageId;
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{milestone::Milestone, MsTangle};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, error, info};
use tokio::sync::mpsc;

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
    bus: &Bus<'static>,
    milestone_cone_updater: &mpsc::UnboundedSender<MilestoneConeUpdaterWorkerEvent>,
) {
    let mut children = vec![message_id];

    while let Some(ref message_id) = children.pop() {
        if tangle.is_solid_message(message_id).await {
            continue;
        }

        if let Some(message) = tangle.get(&message_id).await {
            if tangle.is_solid_message(message.parent1()).await && tangle.is_solid_message(message.parent2()).await {
                // get OTRSI/YTRSI from parents
                let parent1_otsri = tangle.otrsi(message.parent1()).await;
                let parent2_otsri = tangle.otrsi(message.parent2()).await;
                let parent1_ytrsi = tangle.ytrsi(message.parent1()).await;
                let parent2_ytrsi = tangle.ytrsi(message.parent2()).await;

                // get best OTRSI/YTRSI from parents
                // unwrap() is safe because parents are solid which implies that OTRSI/YTRSI values are
                // available.
                let best_otrsi = max(parent1_otsri.unwrap(), parent2_otsri.unwrap());
                let best_ytrsi = min(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

                let mut index = None;

                tangle
                    .update_metadata(&message_id, |metadata| {
                        // OTRSI/YTRSI values need to be set before the solid flag, to ensure that the
                        // MilestoneConeUpdater is aware of all values.
                        metadata.set_otrsi(best_otrsi);
                        metadata.set_ytrsi(best_ytrsi);
                        metadata.solidify();

                        // This is possibly not sufficient as there is no guarantee a milestone has been
                        // validated before being solidified, we then also need
                        // to check when a milestone gets validated if it's
                        // already solid.
                        if metadata.flags().is_milestone() {
                            index = Some(metadata.milestone_index());
                        }
                    })
                    .await;

                if let Some(msg_children) = tangle.get_children(&message_id).await {
                    for child in msg_children {
                        children.push(child);
                    }
                }

                bus.dispatch(MessageSolidified(*message_id));

                tangle
                    .insert_tip(*message_id, *message.parent1(), *message.parent2())
                    .await;

                if let Some(index) = index {
                    // TODO we need to get the milestone from the tangle to dispatch it.
                    // At the time of writing, the tangle only contains an index <-> id mapping.
                    // timestamp is obviously wrong in thr meantime.
                    bus.dispatch(LatestSolidMilestoneChanged {
                        index,
                        milestone: Milestone::new(*message_id, 0),
                    });
                    // TODO we need to get the milestone from the tangle to dispatch it.
                    // At the time of writing, the tangle only contains an index <-> id mapping.
                    // timestamp is obviously wrong in thr meantime.
                    if let Err(e) = milestone_cone_updater
                        .send(MilestoneConeUpdaterWorkerEvent(index, Milestone::new(*message_id, 0)))
                    {
                        error!("Sending message_id to `MilestoneConeUpdater` failed: {:?}.", e);
                    }
                }
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
        vec![TypeId::of::<MilestoneConeUpdaterWorker>(), TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let milestone_cone_updater = node.worker::<MilestoneConeUpdaterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(PropagatorWorkerEvent(message_id)) = receiver.next().await {
                propagate(message_id, &tangle, &*bus, &milestone_cone_updater).await;
            }

            let (_, mut receiver) = receiver.split();
            let receiver = receiver.get_mut();
            let mut count = 0;

            while let Ok(PropagatorWorkerEvent(message_id)) = receiver.try_recv() {
                propagate(message_id, &tangle, &*bus, &milestone_cone_updater).await;
                count += 1;
            }

            debug!("Drained {} message ids.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
