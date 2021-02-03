// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{storage::StorageBackend, MsTangle};

use bee_message::{milestone::MilestoneIndex, solid_entry_point::SolidEntryPoint};
use bee_runtime::{node::Node, worker::Worker};
use bee_snapshot::{SnapshotInfo, SnapshotWorker};
use bee_storage::access::{Fetch, Insert};

use async_trait::async_trait;
use log::{error, warn};
use tokio::time::interval;

use std::{
    any::TypeId,
    convert::Infallible,
    time::{Duration, Instant},
};

pub struct TangleWorker;

#[async_trait]
impl<N: Node> Worker<N> for TangleWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<SnapshotWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        // TODO unwraps
        let tangle = MsTangle::<N::Backend>::new(node.storage());
        node.register_resource(tangle);
        let storage = node.storage();
        let tangle = node.resource::<MsTangle<N::Backend>>();

        let full_sep_rx = node.worker::<SnapshotWorker>().unwrap().full_sep_rx.clone();
        let delta_sep_rx = node.worker::<SnapshotWorker>().unwrap().delta_sep_rx.clone();

        // TODO batch ?

        while let Ok((sep, index)) = full_sep_rx.recv_async().await {
            tangle.add_solid_entry_point(sep, index).await;
            Insert::<SolidEntryPoint, MilestoneIndex>::insert(&*storage, &sep, &index)
                .await
                .unwrap();
        }

        // TODO
        // Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(&*storage)
        //     .await
        //     .unwrap();
        while let Ok((sep, index)) = delta_sep_rx.recv_async().await {
            tangle.add_solid_entry_point(sep, index).await;
            Insert::<SolidEntryPoint, MilestoneIndex>::insert(&*storage, &sep, &index)
                .await
                .unwrap();
        }

        tangle
            .add_solid_entry_point(SolidEntryPoint::null(), MilestoneIndex(0))
            .await;

        // This needs to be done after the streams are emptied.

        let snapshot_info = Fetch::<(), SnapshotInfo>::fetch(&*storage, &()).await.unwrap().unwrap();

        tangle.update_latest_milestone_index(snapshot_info.snapshot_index().into());
        tangle.update_snapshot_index(snapshot_info.snapshot_index().into());
        tangle.update_pruning_index(snapshot_info.pruning_index().into());
        // TODO
        // tangle.add_milestone(config.sep_index().into(), *config.sep_id());

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
