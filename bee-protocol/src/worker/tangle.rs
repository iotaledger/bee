// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MsTangle, worker::storage::StorageWorker, MilestoneIndex};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_message::MessageId;
use bee_snapshot::Snapshot;

use async_trait::async_trait;
use futures::StreamExt;
use log::{error, warn};
use tokio::time::interval;

use std::{
    any::TypeId,
    convert::Infallible,
    time::{Duration, Instant},
};

pub struct TangleWorker;

#[async_trait]
impl<N: Node> Worker<N> for TangleWorker {
    type Config = Snapshot;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<StorageWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let storage = node.storage();
        let tangle = MsTangle::<N::Backend>::new(storage);

        node.register_resource(tangle);

        let tangle = node.resource::<MsTangle<N::Backend>>();

        tangle.update_latest_solid_milestone_index(config.header().sep_index().into());
        tangle.update_latest_milestone_index(config.header().sep_index().into());
        tangle.update_snapshot_index(config.header().sep_index().into());
        tangle.update_pruning_index(config.header().sep_index().into());
        // tangle.add_milestone(config.sep_index().into(), *config.sep_id());

        tangle.add_solid_entry_point(MessageId::null(), MilestoneIndex(0));

        for message_id in config.solid_entry_points() {
            tangle.add_solid_entry_point(*message_id, MilestoneIndex(config.header().sep_index()));
        }

        node.spawn::<Self, _, _>(|shutdown| async move {
            let mut ticker = ShutdownStream::new(shutdown, interval(Duration::from_secs(1)));

            while ticker.next().await.is_some() {
                // println!("Tangle len = {}", tangle.len());
            }
        });

        Ok(Self)
    }

    async fn stop(self, node: &mut N) -> Result<(), Self::Error> {
        let tangle = if let Some(tangle) = node.remove_resource::<MsTangle<N::Backend>>() {
            tangle
        } else {
            warn!(
                "The tangle was still in use by other users when the tangle worker stopped. \
                This is a bug, but not a critical one. From here, we'll revert to polling the \
                tangle until other users are finished with it."
            );

            let poll_start = Instant::now();
            let poll_freq = 20;
            let mut interval = interval(Duration::from_millis(poll_freq));
            loop {
                match node.remove_resource::<MsTangle<N::Backend>>() {
                    Some(tangle) => break tangle,
                    None => {
                        if Instant::now().duration_since(poll_start) > Duration::from_secs(5) {
                            error!(
                                "Tangle shutdown polling period elapsed. The tangle will be dropped \
                            without proper shutdown. This should be considered a bug."
                            );
                            return Ok(());
                        } else {
                            interval.tick().await;
                        }
                    }
                }
            }
        };

        Ok(tangle.shutdown().await)
    }
}
