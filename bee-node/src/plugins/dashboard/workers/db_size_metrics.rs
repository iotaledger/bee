// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::Dashboard, storage::StorageBackend};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};
use bee_tangle::MsTangle;

use futures::StreamExt;
use log::info;
use tokio::time::interval;

use std::time::Duration;

const DB_SIZE_WORKER_INTERVAL_SEC: u64 = 60;

pub(crate) fn db_size_metrics_worker<N>(node: &mut N)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let bus = node.bus();
    let tangle = node.resource::<MsTangle<N::Backend>>();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        info!("Ws `db_size_metrics_worker` running.");

        let mut ticker = ShutdownStream::new(shutdown, interval(Duration::from_secs(DB_SIZE_WORKER_INTERVAL_SEC)));

        while ticker.next().await.is_some() {
            // TODO: replace with storage access once available
            let lsmi = *tangle.get_latest_solid_milestone_index();
            let estimated_total_count = (lsmi - *tangle.get_snapshot_index()) * 12 * DB_SIZE_WORKER_INTERVAL_SEC as u32;
            bus.dispatch(DatabaseSizeMetrics {
                total: estimated_total_count as u64,
                ts: (estimated_total_count as f64 * 0.05) as u64,
            });
        }

        info!("Ws `db_size_metrics_worker` stopped.");
    });
}

#[derive(Clone)]
pub struct DatabaseSizeMetrics {
    pub total: u64,
    pub ts: u64,
}
