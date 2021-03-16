// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::NodeConfig,
    plugins::dashboard::{
        broadcast,
        websocket::{
            responses::{node_status, public_node_status},
            WsUsers,
        },
        Dashboard,
    },
    storage::StorageBackend,
};

use bee_protocol::{PeerManager, ProtocolMetrics};
use bee_rest_api::endpoints::routes::health::is_healthy;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};
use bee_tangle::MsTangle;

use futures::StreamExt;
use log::debug;
use serde::Serialize;
use std::time::Instant;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::time::Duration;

use cap::Cap;
use std::alloc;

#[global_allocator]
pub static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

const NODE_STATUS_METRICS_WORKER_INTERVAL_SEC: u64 = 1;

pub(crate) fn node_status_worker<N>(node: &mut N, users: &WsUsers)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let tangle = node.resource::<MsTangle<N::Backend>>();
    let peer_manager = node.resource::<PeerManager>();
    let node_config = node.resource::<NodeConfig<N::Backend>>();
    let metrics = node.resource::<ProtocolMetrics>();
    let node_info = node.info();
    let users = users.clone();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws PublicNodeStatus/NodeStatus topics handler running.");

        let mut ticker = ShutdownStream::new(
            shutdown,
            IntervalStream::new(interval(Duration::from_secs(NODE_STATUS_METRICS_WORKER_INTERVAL_SEC))),
        );
        let uptime = Instant::now();

        while ticker.next().await.is_some() {
            let public_node_status = PublicNodeStatus {
                snapshot_index: *tangle.get_snapshot_index(),
                pruning_index: *tangle.get_pruning_index(),
                is_healthy: is_healthy(&tangle, &peer_manager).await,
                is_synced: tangle.is_synced(),
            };

            let node_status = NodeStatus {
                version: node_info.version.clone(),
                latest_version: node_info.version.clone(),
                uptime: uptime.elapsed().as_millis() as u64,
                node_id: node_config.node_id.to_string(),
                node_alias: node_config.alias.clone(),
                bech32_hrp: node_config.bech32_hrp.clone(),
                connected_peers_count: 0,
                current_requested_ms: 0,
                request_queue_queued: 0,
                request_queue_pending: 0,
                request_queue_processing: 0,
                request_queue_avg_latency: 0,
                server_metrics: ServerMetrics {
                    all_msgs: metrics.messages_received() as usize,
                    new_msgs: metrics.new_messages() as usize,
                    known_msgs: metrics.known_messages() as usize,
                    invalid_msgs: metrics.invalid_messages() as usize,
                    invalid_req: 0,
                    rec_msg_req: metrics.message_requests_received() as usize,
                    rec_ms_req: metrics.milestone_requests_received() as usize,
                    rec_heartbeat: metrics.heartbeats_received() as usize,
                    sent_msgs: metrics.messages_sent() as usize,
                    sent_msg_req: metrics.message_requests_sent() as usize,
                    sent_ms_req: metrics.milestone_requests_sent() as usize,
                    sent_heartbeat: metrics.heartbeats_sent() as usize,
                    dropped_sent_packets: 0,
                    sent_spam_messages: 0,
                    validated_messages: 0,
                },
                mem: Mem {
                    sys: 0,
                    heap_sys: 0,
                    heap_inuse: ALLOCATOR.allocated(),
                    heap_idle: 0,
                    heap_released: 0,
                    heap_objects: 0,
                    m_span_inuse: 0,
                    m_cache_inuse: 0,
                    stack_sys: 0,
                    num_gc: 0,
                    last_pause_gc: 0,
                },
                caches: Caches {
                    request_queue: RequestQueue { size: 0 },
                    children: Children { size: 0 },
                    milestones: Milestones { size: 0 },
                    messages: Messages { size: 0 },
                    incoming_message_work_units: IncomingMessageWorkUnits { size: 0 },
                },
            };

            broadcast(node_status::forward(node_status), &users).await;
            broadcast(public_node_status::forward(public_node_status), &users).await;
        }

        debug!("Ws PublicNodeStatus/NodeStatus topics handler stopped.");
    });
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicNodeStatus {
    pub snapshot_index: u32,
    pub pruning_index: u32,
    pub is_healthy: bool,
    pub is_synced: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct NodeStatus {
    pub version: String,
    pub latest_version: String,
    pub uptime: u64,
    pub node_id: String,
    pub node_alias: String,
    pub bech32_hrp: String,
    pub connected_peers_count: usize,
    pub current_requested_ms: usize,
    pub request_queue_queued: usize,
    pub request_queue_pending: usize,
    pub request_queue_processing: usize,
    pub request_queue_avg_latency: usize,
    pub server_metrics: ServerMetrics,
    pub mem: Mem,
    pub caches: Caches,
}

#[derive(Clone, Debug, Serialize)]
pub struct ServerMetrics {
    pub all_msgs: usize,
    pub new_msgs: usize,
    pub known_msgs: usize,
    pub invalid_msgs: usize,
    pub invalid_req: usize,
    pub rec_msg_req: usize,
    pub rec_ms_req: usize,
    pub rec_heartbeat: usize,
    pub sent_msgs: usize,
    pub sent_msg_req: usize,
    pub sent_ms_req: usize,
    pub sent_heartbeat: usize,
    pub dropped_sent_packets: usize,
    pub sent_spam_messages: usize,
    pub validated_messages: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Mem {
    pub sys: usize,
    pub heap_sys: usize,
    pub heap_inuse: usize,
    pub heap_idle: usize,
    pub heap_released: usize,
    pub heap_objects: usize,
    pub m_span_inuse: usize,
    pub m_cache_inuse: usize,
    pub stack_sys: usize,
    pub num_gc: usize,
    pub last_pause_gc: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Caches {
    pub request_queue: RequestQueue,
    pub children: Children,
    pub milestones: Milestones,
    pub messages: Messages,
    pub incoming_message_work_units: IncomingMessageWorkUnits,
}

#[derive(Clone, Debug, Serialize)]
pub struct RequestQueue {
    pub size: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Children {
    pub size: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Milestones {
    pub size: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Messages {
    pub size: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct IncomingMessageWorkUnits {
    pub size: usize,
}
