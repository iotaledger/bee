// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MsTangle, worker::storage::StorageWorker, MilestoneIndex};

use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::MessageId;
use bee_snapshot::{SnapshotHeader, SnapshotWorker, SolidEntryPoints};

use async_trait::async_trait;
use log::{error, warn};
use tokio::time::interval;

use std::{
    any::TypeId,
    convert::Infallible,
    ops::Deref,
    time::{Duration, Instant},
};

pub struct TangleWorker;

#[async_trait]
impl<N: Node> Worker<N> for TangleWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<StorageWorker>(), TypeId::of::<SnapshotWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let storage = node.storage();
        let tangle = MsTangle::<N::Backend>::new(storage);
        node.register_resource(tangle);
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let snapshot_header = node.resource::<SnapshotHeader>();
        let seps = node.resource::<SolidEntryPoints>();

        tangle.update_latest_solid_milestone_index(snapshot_header.sep_index().into());
        tangle.update_latest_milestone_index(snapshot_header.sep_index().into());
        tangle.update_snapshot_index(snapshot_header.sep_index().into());
        tangle.update_pruning_index(snapshot_header.sep_index().into());
        // TODO
        // tangle.add_milestone(config.sep_index().into(), *config.sep_id());

        // TODO oh no :(
        for message_id in seps.deref().deref().deref() {
            tangle.add_solid_entry_point(*message_id, MilestoneIndex(snapshot_header.sep_index()));
        }
        tangle.add_solid_entry_point(MessageId::null(), MilestoneIndex(0));

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
