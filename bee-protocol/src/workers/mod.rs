// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod event;
pub mod storage;

mod block;
mod broadcaster;
mod heartbeater;
mod index_updater;
mod metrics;
mod mps;
mod packets;
mod peer;
mod propagator;
mod requester;
mod responder;
mod sender;
mod solidifier;
mod status;

use bee_autopeering::event::EventRx as AutopeeringEventRx;
use bee_gossip::NetworkEventReceiver as NetworkEventRx;
use bee_runtime::node::{Node, NodeBuilder};

use self::peer::PeerManagerConfig;
pub use self::{
    block::{BlockSubmitterError, BlockSubmitterWorker, BlockSubmitterWorkerEvent},
    metrics::MetricsWorker,
    peer::{PeerManager, PeerManagerResWorker},
    requester::{request_block, BlockRequesterWorker, RequestedBlocks, RequestedMilestones},
};
pub(crate) use self::{
    block::{
        HasherWorker, HasherWorkerEvent, MilestonePayloadWorker, PayloadWorker, PayloadWorkerEvent, ProcessorWorker,
        ProcessorWorkerConfig, TaggedDataPayloadWorker, TaggedDataPayloadWorkerEvent, TransactionPayloadWorker,
        UnreferencedBlockInserterWorker, UnreferencedBlockInserterWorkerEvent,
    },
    broadcaster::{BroadcasterWorker, BroadcasterWorkerEvent},
    heartbeater::HeartbeaterWorker,
    index_updater::{IndexUpdaterWorker, IndexUpdaterWorkerEvent},
    mps::MpsWorker,
    peer::{PeerManagerWorker, PeerWorker},
    propagator::{PropagatorWorker, PropagatorWorkerEvent},
    requester::{MilestoneRequesterWorker, MilestoneRequesterWorkerEvent},
    responder::{
        BlockResponderWorker, BlockResponderWorkerEvent, MilestoneResponderWorker, MilestoneResponderWorkerEvent,
    },
    solidifier::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent},
    status::StatusWorker,
};

pub fn init<N: Node>(
    config: config::ProtocolConfig,
    network_id: (String, u64),
    network_events: NetworkEventRx,
    autopeering_events: Option<AutopeeringEventRx>,
    node_builder: N::Builder,
) -> N::Builder
where
    N::Backend: storage::StorageBackend,
{
    node_builder
        .with_worker::<MetricsWorker>()
        .with_worker::<PeerManagerResWorker>()
        .with_worker_cfg::<PeerManagerWorker>(PeerManagerConfig {
            network_rx: network_events,
            peering_rx: autopeering_events,
            network_name: network_id.0,
        })
        .with_worker_cfg::<HasherWorker>(config.clone())
        .with_worker_cfg::<ProcessorWorker>(ProcessorWorkerConfig {
            network_id: network_id.1,
            minimum_pow_score: config.minimum_pow_score,
            rent: config.rent.clone(),
        })
        .with_worker::<BlockResponderWorker>()
        .with_worker::<MilestoneResponderWorker>()
        .with_worker::<BlockRequesterWorker>()
        .with_worker::<MilestoneRequesterWorker>()
        .with_worker::<PayloadWorker>()
        .with_worker::<TransactionPayloadWorker>()
        .with_worker_cfg::<MilestonePayloadWorker>(config.clone())
        .with_worker::<TaggedDataPayloadWorker>()
        .with_worker::<PayloadWorker>()
        .with_worker::<BroadcasterWorker>()
        .with_worker::<PropagatorWorker>()
        .with_worker::<MpsWorker>()
        .with_worker_cfg::<MilestoneSolidifierWorker>(config.workers.milestone_sync_count)
        .with_worker::<IndexUpdaterWorker>()
        .with_worker_cfg::<StatusWorker>(config.workers.status_interval)
        .with_worker::<HeartbeaterWorker>()
        .with_worker::<BlockSubmitterWorker>()
        .with_worker::<UnreferencedBlockInserterWorker>()
}
