// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    protocol::Protocol,
    tangle::MsTangle,
    worker::{MilestoneRequesterWorker, MilestoneSolidifierWorker, RequestedMilestones, TangleWorker},
};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::{channel::oneshot, StreamExt};
use log::{error, info};
use tokio::time::interval;

use std::{any::TypeId, convert::Infallible, time::Duration};

const KICKSTART_INTERVAL_SEC: u64 = 1;

#[derive(Default)]
pub(crate) struct KickstartWorker {}

#[async_trait]
impl<N: Node> Worker<N> for KickstartWorker {
    type Config = (oneshot::Sender<MilestoneIndex>, u32);
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<TangleWorker>(),
            // TODO Temporary until we find a better design for the kickstart
            TypeId::of::<MilestoneSolidifierWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, interval(Duration::from_secs(KICKSTART_INTERVAL_SEC)));

            while receiver.next().await.is_some() {
                let next_ms = *tangle.get_latest_solid_milestone_index() + 1;
                let latest_ms = *tangle.get_latest_milestone_index();

                if !Protocol::get().peer_manager.peers.is_empty() && next_ms + config.1 < latest_ms {
                    Protocol::request_milestone(
                        &tangle,
                        &milestone_requester,
                        &*requested_milestones,
                        MilestoneIndex(next_ms),
                        None,
                    );
                    if config.0.send(MilestoneIndex(next_ms)).is_err() {
                        error!("Could not set first non-solid milestone");
                    }

                    for index in next_ms..(next_ms + config.1) {
                        Protocol::request_milestone(
                            &tangle,
                            &milestone_requester,
                            &*requested_milestones,
                            MilestoneIndex(index),
                            None,
                        );
                    }
                    break;
                }
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
