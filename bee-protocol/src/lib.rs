// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::module_inception)]
#![allow(clippy::unit_arg)]

pub mod config;
pub mod event;
pub mod milestone;
pub mod tangle;

mod helper;
mod metrics;
mod packet;
mod peer;
mod sender;
mod worker;

pub use metrics::ProtocolMetrics;
pub use milestone::{Milestone, MilestoneIndex};
pub use worker::{
    MessageSubmitterError, MessageSubmitterWorker, MessageSubmitterWorkerEvent, MetricsWorker, TangleWorker,
};

use crate::{
    config::ProtocolConfig,
    peer::{Peer, PeerManager},
    sender::Sender,
    tangle::MsTangle,
    worker::{
        BroadcasterWorker, HasherWorker, HeartbeaterWorker, IndexationPayloadWorker, MessageRequesterWorker,
        MessageResponderWorker, MessageValidatorWorker, MilestoneConeUpdaterWorker, MilestonePayloadWorker,
        MilestoneRequesterWorker, MilestoneResponderWorker, MilestoneSolidifierWorker, MpsWorker, PeerManagerWorker,
        PeerWorker, ProcessorWorker, PropagatorWorker, RequestedMilestones, StatusWorker, TipPoolCleanerWorker,
        TransactionPayloadWorker,
    },
};

use bee_common_pt2::node::{Node, NodeBuilder};
use bee_network::{Multiaddr, PeerId};

use futures::channel::oneshot;
use tokio::{sync::mpsc, task::spawn};

use std::sync::Arc;

pub fn init<N: Node>(config: ProtocolConfig, network_id: u64, node_builder: N::Builder) -> N::Builder {
    node_builder
        .with_worker::<TangleWorker>()
        .with_worker::<MetricsWorker>()
        .with_worker::<PeerManagerWorker>()
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

pub async fn register<N: Node>(
    node: &N,
    id: PeerId,
    address: Multiaddr,
) -> (mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>) {
    // TODO check if not already added ?

    let peer = Arc::new(Peer::new(id, address));

    let (receiver_tx, receiver_rx) = mpsc::unbounded_channel();
    let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

    let tangle = node.resource::<MsTangle<N::Backend>>();
    let requested_milestones = node.resource::<RequestedMilestones>();
    let metrics = node.resource::<ProtocolMetrics>();
    let peer_manager = node.resource::<PeerManager>();

    peer_manager.add(peer.clone()).await;

    spawn(
        PeerWorker::new(
            peer,
            metrics,
            peer_manager,
            node.worker::<HasherWorker>().unwrap().tx.clone(),
            node.worker::<MessageResponderWorker>().unwrap().tx.clone(),
            node.worker::<MilestoneResponderWorker>().unwrap().tx.clone(),
            node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone(),
        )
        .run(tangle.clone(), requested_milestones, receiver_rx, receiver_shutdown_rx),
    );

    (receiver_tx, receiver_shutdown_tx)
}

pub async fn unregister<N: Node>(node: &N, id: PeerId) {
    node.resource::<PeerManager>().remove(&id).await;
}
