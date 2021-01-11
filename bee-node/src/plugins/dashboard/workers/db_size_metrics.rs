// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::Dashboard, storage::StorageBackend};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use futures::StreamExt;
use log::debug;
use tokio::time::interval;

use std::time::Duration;

const DB_SIZE_METRICS_WORKER_INTERVAL_SEC: u64 = 1;

pub(crate) fn db_size_metrics_worker<N>(node: &mut N)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let bus = node.bus();
    let storage = node.storage();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws `db_size_metrics_worker` running.");

        let mut ticker = ShutdownStream::new(
            shutdown,
            interval(Duration::from_secs(DB_SIZE_METRICS_WORKER_INTERVAL_SEC)),
        );

        use bee_storage::backend::StorageBackend;
        while ticker.next().await.is_some() {
            // TODO: replace with storage access once available
            // Storage should be
            let size = storage.size().await.unwrap().unwrap() as u64;
            bus.dispatch(DatabaseSizeMetrics {
                total: size,
                // replace with appropriate storage function
                ts: size,
            });
        }

        debug!("Ws `db_size_metrics_worker` stopped.");
    });
}

#[derive(Clone)]
pub struct DatabaseSizeMetrics {
    pub total: u64,
    pub ts: u64,
}
