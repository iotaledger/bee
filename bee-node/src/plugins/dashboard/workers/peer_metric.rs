// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::{broadcast, websocket::WsUsers, Dashboard},
    storage::StorageBackend,
};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use futures::StreamExt;
use log::debug;
use serde::Serialize;

use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use crate::plugins::dashboard::websocket::responses::peer_metric;
use bee_protocol::PeerManager;

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
        debug!("Ws NodeStatus topic handler running.");

        let mut ticker = ShutdownStream::new(
            shutdown,
            IntervalStream::new(interval(Duration::from_secs(NODE_STATUS_METRICS_WORKER_INTERVAL_SEC))),
        );

        while ticker.next().await.is_some() {
            for peer in peer_manager.get_all().await {
                println!("peers are present");
                let peer_metric_dto = PeerMetric {
                    identity: peer.id().to_string(),
                    alias: peer.alias().to_string(),
                    origin_addr: peer.address().to_string(),
                    connection_origin: 0,
                    protocol_version: 0,
                    bytes_read: 0,
                    bytes_written: 0,
                    heartbeat: PeerHeartbeat {
                        solid_milestone_index: 0,
                        pruned_milestone_index: 0,
                        latest_milestone_index: 0,
                        connected_neighbors: 0,
                        synced_neighbors: 0,
                    },
                    info: PeerInfo {
                        address: peer.address().to_string(),
                        port: 0,
                        domain: String::from(""),
                        numberOfAllTransactions: 0,
                        numberOfNewTransactions: 0,
                        numberOfKnownTransactions: 0,
                        numberOfReceivedTransactionReq: 0,
                        numberOfReceivedMilestoneReq: 0,
                        numberOfReceivedHeartbeats: 0,
                        numberOfSentTransactions: 0,
                        numberOfSentTransactionsReq: 0,
                        numberOfSentMilestoneReq: 0,
                        numberOfSentHeartbeats: 0,
                        numberOfDroppedSentPackets: 0,
                        connectionType: String::from(""),
                        autopeeringId: String::from(""),
                        connected: false,
                    },
                    connected: false,
                    ts: 0,
                };

                broadcast(peer_metric::forward(peer_metric_dto), &users).await;
            }
        }

        debug!("Ws NodeStatus topic handler stopped.");
    });
}

#[derive(Clone, Debug, Serialize)]
pub struct PeerMetric {
    pub identity: String,
    pub alias: String,
    pub origin_addr: String,
    pub connection_origin: usize,
    pub protocol_version: usize,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub heartbeat: PeerHeartbeat,
    pub info: PeerInfo,
    pub connected: bool,
    pub ts: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PeerHeartbeat {
    pub solid_milestone_index: usize,
    pub pruned_milestone_index: usize,
    pub latest_milestone_index: usize,
    pub connected_neighbors: usize,
    pub synced_neighbors: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct PeerInfo {
    pub address: String,
    pub port: size,
    pub domain: String,
    pub numberOfAllTransactions: usize,
    pub numberOfNewTransactions: usize,
    pub numberOfKnownTransactions: usize,
    pub numberOfReceivedTransactionReq: usize,
    pub numberOfReceivedMilestoneReq: usize,
    pub numberOfReceivedHeartbeats: usize,
    pub numberOfSentTransactions: usize,
    pub numberOfSentTransactionsReq: usize,
    pub numberOfSentMilestoneReq: usize,
    pub numberOfSentHeartbeats: usize,
    pub numberOfDroppedSentPackets: usize,
    pub connectionType: String,
    pub autopeeringId: String,
    pub connected: bool,
}
