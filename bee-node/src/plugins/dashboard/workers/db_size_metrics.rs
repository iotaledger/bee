// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::{
        broadcast,
        websocket::{responses::database_size_metrics, WsUsers},
        Dashboard,
    },
    storage::StorageBackend,
};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};
use bee_storage::backend::StorageBackend as _;

use futures::StreamExt;
use log::debug;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::time::Duration;

const DB_SIZE_METRICS_WORKER_INTERVAL_SEC: u64 = 60;

pub(crate) fn db_size_metrics_worker<N>(node: &mut N, users: &WsUsers)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let storage = node.storage();
    let users = users.clone();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws DatabaseSizeMetrics topic handler running.");

        let mut ticker = ShutdownStream::new(
            shutdown,
            IntervalStream::new(interval(Duration::from_secs(DB_SIZE_METRICS_WORKER_INTERVAL_SEC))),
        );

        while ticker.next().await.is_some() {
            let metrics = DatabaseSizeMetrics {
                total: storage.size().await.unwrap().unwrap() as u64,
                ts: 0, // replace with appropriate storage function
            };
            broadcast(database_size_metrics::forward(metrics), &users).await;
        }

        debug!("Ws DatabaseSizeMetrics topic handler stopped.");
    });
}

#[derive(Clone)]
pub struct DatabaseSizeMetrics {
    pub total: u64,
    pub ts: u64,
}
