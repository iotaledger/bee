// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::module_inception)]
#![allow(clippy::unit_arg)]

pub mod config;
pub mod event;
pub mod milestone;
pub mod storage;
pub mod tangle;

mod helper;
mod metrics;
mod packet;
mod peer;
mod sender;
mod worker;

pub use metrics::ProtocolMetrics;
pub use milestone::{Milestone, MilestoneIndex};
pub use storage::Backend;
pub use worker::{
    MessageSubmitterError, MessageSubmitterWorker, MessageSubmitterWorkerEvent, MetricsWorker, TangleWorker,
};

use crate::{
    config::ProtocolConfig,
    sender::Sender,
    worker::{
        BroadcasterWorker, HasherWorker, HeartbeaterWorker, IndexationPayloadWorker, MessageRequesterWorker,
        MessageResponderWorker, MessageValidatorWorker, MilestoneConeUpdaterWorker, MilestonePayloadWorker,
        MilestoneRequesterWorker, MilestoneResponderWorker, MilestoneSolidifierWorker, MpsWorker, PeerManagerResWorker,
        PeerManagerWorker, ProcessorWorker, PropagatorWorker, StatusWorker, TipPoolCleanerWorker,
        TransactionPayloadWorker,
    },
};

use bee_common_pt2::node::{Node, NodeBuilder};
use bee_network::NetworkListener;

pub fn init<N: Node>(
    config: ProtocolConfig,
    network_id: u64,
    events: NetworkListener,
    node_builder: N::Builder,
) -> N::Builder
where
    N::Backend: Backend,
{
    node_builder
        .with_worker::<TangleWorker>()
        .with_worker::<MetricsWorker>()
        .with_worker::<PeerManagerResWorker>()
        .with_worker_cfg::<PeerManagerWorker>(events)
        .with_worker_cfg::<HasherWorker>(config.workers.message_worker_cache)
        .with_worker_cfg::<ProcessorWorker>((config.clone(), network_id))
        .with_worker::<MessageResponderWorker>()
        .with_worker::<MilestoneResponderWorker>()
        .with_worker::<MessageRequesterWorker>()
        .with_worker::<MilestoneRequesterWorker>()
        .with_worker::<TransactionPayloadWorker>()
        .with_worker_cfg::<MilestonePayloadWorker>(config.clone())
        .with_worker::<IndexationPayloadWorker>()
        .with_worker::<BroadcasterWorker>()
        .with_worker::<MessageValidatorWorker>()
        .with_worker::<PropagatorWorker>()
        .with_worker::<MpsWorker>()
        .with_worker_cfg::<MilestoneSolidifierWorker>(config.workers.ms_sync_count)
        .with_worker::<MilestoneConeUpdaterWorker>()
        .with_worker::<TipPoolCleanerWorker>()
        .with_worker_cfg::<StatusWorker>(config.workers.status_interval)
        .with_worker::<HeartbeaterWorker>()
        .with_worker::<MessageSubmitterWorker>()
}
