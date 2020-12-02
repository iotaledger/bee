// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Milestone, MilestoneIndex};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::{channel::oneshot, stream::StreamExt};
use log::{error, info, warn};

use crate::{
    event::LatestSolidMilestoneChanged,
    worker::{MilestoneConeUpdaterWorker, MilestoneConeUpdaterWorkerEvent},
};
use bee_common::event::Bus;
use std::{any::TypeId, convert::Infallible};

pub(crate) struct SolidMilestoneAnnouncerWorkerEvent {
    pub(crate) milestone_message_id: MessageId,
    pub(crate) propagator_waiter: oneshot::Receiver<()>,
    pub(crate) milestone_validator_waiter: oneshot::Receiver<MilestoneIndex>,
}

pub(crate) struct SolidMilestoneAnnouncerWorker {
    pub(crate) tx: flume::Sender<SolidMilestoneAnnouncerWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for SolidMilestoneAnnouncerWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MilestoneConeUpdaterWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let cone_updater = node.worker::<MilestoneConeUpdaterWorker>().unwrap().tx.clone();
        let bus = node.resource::<Bus>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(event) = receiver.next().await {
                let event: SolidMilestoneAnnouncerWorkerEvent = event;

                if let Err(e) = event.propagator_waiter.await {
                    error!("Waiting for propagator failed: {:?}.", e);
                }

                match event.milestone_validator_waiter.await {
                    Ok(index) => {
                        if let Err(e) = cone_updater.send(MilestoneConeUpdaterWorkerEvent {
                            milestone_message_id: event.milestone_message_id,
                            milestone_index: index,
                        }) {
                            error!("Sending milestone index to: {:?}.", e);
                        }
                        bus.dispatch(LatestSolidMilestoneChanged(Milestone {
                            message_id: event.milestone_message_id,
                            index,
                        }));
                    }
                    Err(e) => error!("Waiting for milestone validator failed: {:?}.", e),
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
