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
    MessageSubmitterError, MessageSubmitterWorker, MessageSubmitterWorkerEvent, StorageWorker, TangleWorker,
};

use crate::{
    config::ProtocolConfig,
    event::{LatestMilestoneChanged, LatestSolidMilestoneChanged},
    peer::{Peer, PeerManager},
    sender::Sender,
    tangle::MsTangle,
    worker::{
        BroadcasterWorker, HasherWorker, HeartbeaterWorker, IndexationPayloadWorker, KickstartWorker,
        MessageRequesterWorker, MessageResponderWorker, MessageValidatorWorker, MetricsWorker,
        MilestoneConeUpdaterWorker, MilestonePayloadWorker, MilestoneRequesterWorker, MilestoneResponderWorker,
        MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, MpsWorker, PeerManagerWorker, PeerWorker,
        ProcessorWorker, PropagatorWorker, RequestedMilestones, StatusWorker, TipPoolCleanerWorker,
    },
};

use bee_common::event::Bus;
use bee_common_pt2::node::{Node, NodeBuilder};
use bee_network::{Multiaddr, NetworkController, PeerId};
use bee_storage::storage::Backend;

use futures::channel::oneshot;
use log::{debug, error, info};
use tokio::{sync::mpsc, task::spawn};

use std::sync::Arc;

pub fn init<N: Node>(
    config: ProtocolConfig,
    database_config: <N::Backend as Backend>::Config,
    network_id: u64,
    node_builder: N::Builder,
) -> N::Builder {
    let (ms_send, ms_recv) = oneshot::channel();

    node_builder
        .with_worker_cfg::<StorageWorker>(database_config)
        .with_worker::<TangleWorker>()
        .with_worker::<MetricsWorker>()
        .with_worker::<PeerManagerWorker>()
        .with_worker_cfg::<HasherWorker>(config.workers.message_worker_cache)
        .with_worker_cfg::<ProcessorWorker>((config.clone(), network_id))
        .with_worker::<MessageResponderWorker>()
        .with_worker::<MilestoneResponderWorker>()
        .with_worker::<MessageRequesterWorker>()
        .with_worker::<MilestoneRequesterWorker>()
        .with_worker_cfg::<MilestonePayloadWorker>(config.clone())
        .with_worker::<IndexationPayloadWorker>()
        .with_worker::<BroadcasterWorker>()
        .with_worker::<MessageValidatorWorker>()
        .with_worker::<PropagatorWorker>()
        .with_worker::<MpsWorker>()
        .with_worker_cfg::<KickstartWorker>((ms_send, config.workers.ms_sync_count))
        .with_worker_cfg::<MilestoneSolidifierWorker>(ms_recv)
        .with_worker::<MilestoneConeUpdaterWorker>()
        .with_worker::<TipPoolCleanerWorker>()
        .with_worker_cfg::<StatusWorker>(config.workers.status_interval)
        .with_worker::<HeartbeaterWorker>()
        .with_worker::<MessageSubmitterWorker>()
}

pub fn events<N: Node>(node: &N, config: ProtocolConfig) {
    let tangle = node.resource::<MsTangle<N::Backend>>().into_weak();
    let network = node.resource::<NetworkController>(); // TODO: Use a weak handle?
    let peer_manager = node.resource::<PeerManager>();
    let metrics = node.resource::<ProtocolMetrics>();

    node.resource::<Bus>()
        .add_listener::<(), _, _>(move |latest_milestone: &LatestMilestoneChanged| {
            info!(
                "New milestone {} {}.",
                *latest_milestone.index, latest_milestone.milestone.message_id
            );
            if let Some(tangle) = tangle.upgrade() {
                tangle.update_latest_milestone_index(latest_milestone.index);

                helper::broadcast_heartbeat(
                    &peer_manager,
                    &network,
                    &metrics,
                    tangle.get_latest_solid_milestone_index(),
                    tangle.get_pruning_index(),
                    latest_milestone.index,
                );
            }
        });

    // node.resource::<Bus>().add_listener(|latest_solid_milestone: &LatestSolidMilestoneChanged| {
    //     // TODO block_on ?
    //     // TODO uncomment on Chrysalis Pt1.
    //     block_on(Protocol::broadcast_heartbeat(
    //         tangle.get_latest_solid_milestone_index(),
    //         tangle.get_pruning_index(),
    //     ));
    // });

    let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();
    let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();

    let tangle = node.resource::<MsTangle<N::Backend>>().into_weak();
    let network = node.resource::<NetworkController>(); // TODO: Use a weak handle?
    let requested_milestones = node.resource::<RequestedMilestones>();
    let peer_manager = node.resource::<PeerManager>();
    let metrics = node.resource::<ProtocolMetrics>();

    node.resource::<Bus>()
        .add_listener::<(), _, _>(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
            if let Some(tangle) = tangle.upgrade() {
                debug!("New solid milestone {}.", *latest_solid_milestone.index);
                tangle.update_latest_solid_milestone_index(latest_solid_milestone.index);

                let ms_sync_count = config.workers.ms_sync_count;
                let next_ms = latest_solid_milestone.index + MilestoneIndex(ms_sync_count);

                if tangle.contains_milestone(next_ms) {
                    if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(next_ms)) {
                        error!("Sending solidification event failed: {}", e);
                    }
                } else {
                    helper::request_milestone(&tangle, &milestone_requester, &*requested_milestones, next_ms, None);
                }

                helper::broadcast_heartbeat(
                    &peer_manager,
                    &network,
                    &metrics,
                    latest_solid_milestone.index,
                    tangle.get_pruning_index(),
                    tangle.get_latest_milestone_index(),
                );
            }
        });
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
