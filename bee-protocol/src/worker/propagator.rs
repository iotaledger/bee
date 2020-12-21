// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::{LatestSolidMilestoneChanged, MessageSolidified},
    milestone::Milestone,
    tangle::MsTangle,
    worker::{
        milestone_cone_updater::{MilestoneConeUpdaterWorker, MilestoneConeUpdaterWorkerEvent},
        MessageValidatorWorker, MessageValidatorWorkerEvent, TangleWorker,
    },
};

use bee_common::{event::Bus, shutdown_stream::ShutdownStream};
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{error, info, warn};
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

#[async_trait]
impl<N: Node> Worker<N> for PropagatorWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<MessageValidatorWorker>(),
            TypeId::of::<MilestoneConeUpdaterWorker>(),
            TypeId::of::<TangleWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let message_validator = node.worker::<MessageValidatorWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<MilestoneConeUpdaterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let bus = node.resource::<Bus>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(PropagatorWorkerEvent(hash)) = receiver.next().await {
                let mut children = vec![hash];

                while let Some(ref hash) = children.pop() {
                    if tangle.is_solid_message(hash) {
                        continue;
                    }

                    if let Some(message) = tangle.get(&hash).await {
                        if tangle.is_solid_message(message.parent1()) && tangle.is_solid_message(message.parent2()) {
                            // get OTRSI/YTRSI from parents
                            let parent1_otsri = tangle.otrsi(message.parent1());
                            let parent2_otsri = tangle.otrsi(message.parent2());
                            let parent1_ytrsi = tangle.ytrsi(message.parent1());
                            let parent2_ytrsi = tangle.ytrsi(message.parent2());

                            // get best OTRSI/YTRSI from parents
                            // unwrap() is safe because parents are solid which implies that OTRSI/YTRSI values are
                            // available.
                            let best_otrsi = max(parent1_otsri.unwrap(), parent2_otsri.unwrap());
                            let best_ytrsi = min(parent1_ytrsi.unwrap(), parent2_ytrsi.unwrap());

                            let mut index = None;

                            tangle.update_metadata(&hash, |metadata| {
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
                            });

                            for child in tangle.get_children(&hash) {
                                children.push(child);
                            }

                            bus.dispatch(MessageSolidified(*hash));

                            if let Err(e) = message_validator.send(MessageValidatorWorkerEvent(*hash)) {
                                warn!("Failed to send hash to message validator: {:?}.", e);
                            }

                            if let Some(index) = index {
                                // TODO we need to get the milestone from the tangle to dispatch it.
                                // At the time of writing, the tangle only contains an index <-> id mapping.
                                // timestamp is obviously wrong in thr meantime.
                                bus.dispatch(LatestSolidMilestoneChanged {
                                    index,
                                    milestone: Milestone {
                                        message_id: *hash,
                                        timestamp: 0,
                                    },
                                });
                                // TODO we need to get the milestone from the tangle to dispatch it.
                                // At the time of writing, the tangle only contains an index <-> id mapping.
                                // timestamp is obviously wrong in thr meantime.
                                if let Err(e) = milestone_cone_updater.send(MilestoneConeUpdaterWorkerEvent(
                                    index,
                                    Milestone {
                                        message_id: *hash,
                                        timestamp: 0,
                                    },
                                )) {
                                    error!("Sending hash to `MilestoneConeUpdater` failed: {:?}.", e);
                                }
                            }
                        }
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
