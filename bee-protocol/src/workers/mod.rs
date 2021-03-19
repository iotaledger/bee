// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod event;
pub mod storage;

mod broadcaster;
mod heartbeater;
mod helper;
mod index_updater;
mod message;
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
mod tip_pool_cleaner;

pub(crate) use broadcaster::{BroadcasterWorker, BroadcasterWorkerEvent};
pub(crate) use heartbeater::HeartbeaterWorker;
pub(crate) use index_updater::{IndexUpdaterWorker, IndexUpdaterWorkerEvent};
pub(crate) use message::{
    HasherWorker, HasherWorkerEvent, IndexationPayloadWorker, IndexationPayloadWorkerEvent, MilestonePayloadWorker,
    PayloadWorker, PayloadWorkerEvent, ProcessorWorker, TransactionPayloadWorker, UnconfirmedMessageInserterWorker,
    UnconfirmedMessageInserterWorkerEvent,
};
pub use message::{MessageSubmitterError, MessageSubmitterWorker, MessageSubmitterWorkerEvent};
pub use metrics::MetricsWorker;
pub(crate) use mps::MpsWorker;
pub use peer::{PeerManager, PeerManagerResWorker};
pub(crate) use peer::{PeerManagerWorker, PeerWorker};
pub(crate) use propagator::{PropagatorWorker, PropagatorWorkerEvent};
pub(crate) use requester::{
    MessageRequesterWorker, MessageRequesterWorkerEvent, MilestoneRequesterWorker, MilestoneRequesterWorkerEvent,
    RequestedMessages, RequestedMilestones,
};
pub(crate) use responder::{
    MessageResponderWorker, MessageResponderWorkerEvent, MilestoneResponderWorker, MilestoneResponderWorkerEvent,
};
pub(crate) use solidifier::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent};
pub(crate) use status::StatusWorker;
pub(crate) use tip_pool_cleaner::TipPoolCleanerWorker;

use bee_network::node::NetworkListener;
use bee_runtime::node::{Node, NodeBuilder};

pub fn init<N: Node>(
    config: config::ProtocolConfig,
    network_id: u64,
    events: NetworkListener,
    node_builder: N::Builder,
) -> N::Builder
where
    N::Backend: storage::StorageBackend,
{
    node_builder
        .with_worker::<MetricsWorker>()
        .with_worker::<PeerManagerResWorker>()
        .with_worker_cfg::<PeerManagerWorker>(events)
        .with_worker_cfg::<HasherWorker>(config.workers.message_worker_cache)
        .with_worker_cfg::<ProcessorWorker>((config.clone(), network_id))
        .with_worker::<MessageResponderWorker>()
        .with_worker::<MilestoneResponderWorker>()
        .with_worker::<MessageRequesterWorker>()
        .with_worker::<MilestoneRequesterWorker>()
        .with_worker::<PayloadWorker>()
        .with_worker::<TransactionPayloadWorker>()
        .with_worker_cfg::<MilestonePayloadWorker>(config.clone())
        .with_worker::<IndexationPayloadWorker>()
        .with_worker::<PayloadWorker>()
        .with_worker::<BroadcasterWorker>()
        .with_worker::<PropagatorWorker>()
        .with_worker::<MpsWorker>()
        .with_worker_cfg::<MilestoneSolidifierWorker>(config.workers.ms_sync_count)
        .with_worker::<IndexUpdaterWorker>()
        .with_worker::<TipPoolCleanerWorker>()
        .with_worker_cfg::<StatusWorker>(config.workers.status_interval)
        .with_worker::<HeartbeaterWorker>()
        .with_worker::<MessageSubmitterWorker>()
        .with_worker::<UnconfirmedMessageInserterWorker>()
}
