// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod event;
pub mod storage;

mod broadcaster;
mod heartbeater;
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

use bee_autopeering::event::EventRx as AutopeeringEventRx;
use bee_gossip::NetworkEventReceiver as NetworkEventRx;
use bee_runtime::{
    node::{Node, NodeBuilder},
    worker::Worker,
};

use self::peer::PeerManagerConfig;
pub(crate) use self::{
    broadcaster::{BroadcasterWorker, BroadcasterWorkerEvent},
    heartbeater::HeartbeaterWorker,
    index_updater::{IndexUpdaterWorker, IndexUpdaterWorkerEvent},
    message::{
        HasherWorker, HasherWorkerEvent, IndexationPayloadWorker, IndexationPayloadWorkerEvent, MilestonePayloadWorker,
        PayloadWorker, PayloadWorkerEvent, ProcessorWorker, TransactionPayloadWorker,
        UnreferencedMessageInserterWorker, UnreferencedMessageInserterWorkerEvent,
    },
    mps::MpsWorker,
    peer::{PeerManagerWorker, PeerWorker},
    propagator::{PropagatorWorker, PropagatorWorkerEvent},
    requester::{MilestoneRequesterWorker, MilestoneRequesterWorkerEvent},
    responder::{
        MessageResponderWorker, MessageResponderWorkerEvent, MilestoneResponderWorker, MilestoneResponderWorkerEvent,
    },
    solidifier::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent},
    status::StatusWorker,
};
pub use self::{
    message::{MessageSubmitterError, MessageSubmitterWorker, MessageSubmitterWorkerEvent},
    metrics::{MetricsConfig, MetricsConfigBuilder, MetricsWorker},
    peer::{PeerManager, PeerManagerResWorker},
    requester::{request_message, MessageRequesterWorker, RequestedMessages, RequestedMilestones},
};

pub fn init<N: Node>(
    config: config::ProtocolConfig,
    network_id: (String, u64),
    network_events: NetworkEventRx,
    autopeering_events: Option<AutopeeringEventRx>,
    node_builder: N::Builder,
    metrics_config: <MetricsWorker as Worker<N>>::Config,
) -> N::Builder
where
    N::Backend: storage::StorageBackend,
{
    node_builder
        .with_worker_cfg::<MetricsWorker>(metrics_config)
        .with_worker::<PeerManagerResWorker>()
        .with_worker_cfg::<PeerManagerWorker>(PeerManagerConfig {
            network_rx: network_events,
            peering_rx: autopeering_events,
            network_name: network_id.0,
        })
        .with_worker_cfg::<HasherWorker>(config.clone())
        .with_worker_cfg::<ProcessorWorker>(network_id.1)
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
        .with_worker_cfg::<MilestoneSolidifierWorker>(config.workers.milestone_sync_count)
        .with_worker::<IndexUpdaterWorker>()
        .with_worker_cfg::<StatusWorker>(config.workers.status_interval)
        .with_worker::<HeartbeaterWorker>()
        .with_worker::<MessageSubmitterWorker>()
        .with_worker::<UnreferencedMessageInserterWorker>()
}
