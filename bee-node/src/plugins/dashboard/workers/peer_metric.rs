// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::{
        broadcast,
        websocket::{responses::peer_metric, WsUsers},
        Dashboard,
    },
    storage::StorageBackend,
};

use bee_protocol::PeerManager;
use bee_rest_api::types::{dtos::PeerDto, responses::PeersResponse};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use futures::StreamExt;
use log::debug;

use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::time::Duration;

const NODE_STATUS_METRICS_WORKER_INTERVAL_SEC: u64 = 1;

pub(crate) fn peer_metric_worker<N>(node: &mut N, users: &WsUsers)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let peer_manager = node.resource::<PeerManager>();
    let users = users.clone();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws PeerMetrics topic handler running.");

        let mut ticker = ShutdownStream::new(
            shutdown,
            IntervalStream::new(interval(Duration::from_secs(NODE_STATUS_METRICS_WORKER_INTERVAL_SEC))),
        );

        while ticker.next().await.is_some() {
            let mut peers_dtos = Vec::new();
            for peer in peer_manager.get_all().await {
                peers_dtos.push(PeerDto::from(peer.as_ref()));
            }
            broadcast(peer_metric::forward(PeersResponse(peers_dtos)), &users).await;
        }

        debug!("Ws PeerMetrics topic handler stopped.");
    });
}
