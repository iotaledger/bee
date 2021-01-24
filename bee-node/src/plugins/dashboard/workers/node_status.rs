// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::NodeConfig,
    constants::{BEE_GIT_COMMIT, BEE_VERSION},
    plugins::dashboard::{
        broadcast,
        websocket::{responses::node_status, WsUsers},
        Dashboard,
    },
    storage::StorageBackend,
};

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
    let node_config = node.resource::<NodeConfig<N::Backend>>();
    let peering_config = node_config.peering.clone();
    let users = users.clone();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws NodeStatus topic handler running.");

        let mut ticker = ShutdownStream::new(
            shutdown,
            IntervalStream::new(interval(Duration::from_secs(NODE_STATUS_METRICS_WORKER_INTERVAL_SEC))),
        );
        let uptime = Instant::now();
        let version = if BEE_GIT_COMMIT.is_empty() {
            BEE_VERSION.to_owned()
        } else {
            BEE_VERSION.to_owned() + "-" + &BEE_GIT_COMMIT[0..7]
        };

        while ticker.next().await.is_some() {
            let status = NodeStatus {
                snapshot_index: *tangle.get_snapshot_index(),
                pruning_index: *tangle.get_pruning_index(),
                is_healthy: tangle.is_healthy().await,
                is_synced: tangle.is_synced(),
                version: version.clone(),
                latest_version: version.clone(),
                uptime: uptime.elapsed().as_millis() as u64,
                autopeering_id: peering_config.peer_id.to_string(),
                node_alias: node_config.alias.clone(),
                bech32_hrp: node_config.bech32_hrp.clone(),
                connected_peers_count: 0,
                current_requested_ms: 0,
                request_queue_queued: 0,
                request_queue_pending: 0,
                request_queue_processing: 0,
                request_queue_avg_latency: 0,
                server_metrics: ServerMetrics {
                    all_msgs: 0,
                    new_msgs: 0,
                    known_msgs: 0,
                    invalid_msgs: 0,
                    invalid_req: 0,
                    rec_msg_req: 0,
                    rec_ms_req: 0,
                    rec_heartbeat: 0,
                    sent_msgs: 0,
                    sent_msg_req: 0,
                    sent_ms_req: 0,
                    sent_heartbeat: 0,
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
            broadcast(node_status::forward(status), &users).await;
        }

        debug!("Ws NodeStatus topic handler stopped.");
    });
}

#[derive(Clone, Debug, Serialize)]
pub struct NodeStatus {
    pub snapshot_index: u32,
    pub pruning_index: u32,
    pub is_healthy: bool,
    pub is_synced: bool,
    pub version: String,
    pub latest_version: String,
    pub uptime: u64,
    pub autopeering_id: String,
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
