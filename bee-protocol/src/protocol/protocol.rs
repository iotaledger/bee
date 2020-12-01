// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ProtocolConfig,
    event::{LatestMilestoneChanged, LatestSolidMilestoneChanged},
    milestone::MilestoneIndex,
    peer::{Peer, PeerManager},
    protocol::ProtocolMetrics,
    tangle::MsTangle,
    worker::{
        BroadcasterWorker, HasherWorker, HeartbeaterWorker, KickstartWorker, MessageRequesterWorker,
        MessageResponderWorker, MessageSubmitterWorker, MessageValidatorWorker, MilestoneConeUpdaterWorker,
        MilestoneRequesterWorker, MilestoneResponderWorker, MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent,
        MilestoneValidatorWorker, MpsWorker, PeerWorker, ProcessorWorker, PropagatorWorker, RequestedMilestones,
        StatusWorker, StorageWorker, TangleWorker, TipPoolCleanerWorker,
    },
};

use bee_common::{
    event::Bus,
    node::{Node, NodeBuilder},
};
use bee_network::{Multiaddr, Network, PeerId};
use bee_snapshot::Snapshot;
use bee_storage::storage::Backend;

use futures::channel::oneshot;
use log::{debug, error, info};
use tokio::task::spawn;

use std::sync::Arc;

static PROTOCOL: spin::RwLock<Option<&'static Protocol>> = spin::RwLock::new(None);

pub struct Protocol {
    pub(crate) metrics: ProtocolMetrics,
    pub(crate) peer_manager: PeerManager,
}

impl Protocol {
    pub fn init<N: Node>(
        config: ProtocolConfig,
        database_config: <N::Backend as Backend>::Config,
        snapshot: Snapshot,
        network_id: u64,
        node_builder: N::Builder,
    ) -> N::Builder {
        let protocol = Protocol {
            metrics: ProtocolMetrics::new(),
            peer_manager: PeerManager::new(),
        };

        *PROTOCOL.write() = Some(Box::leak(Box::new(protocol)));

        let (ms_send, ms_recv) = oneshot::channel();

        node_builder
            .with_worker_cfg::<StorageWorker>(database_config)
            .with_worker_cfg::<TangleWorker>(snapshot)
            .with_worker_cfg::<HasherWorker>(config.workers.message_worker_cache)
            .with_worker_cfg::<ProcessorWorker>((config.clone(), network_id))
            .with_worker::<MessageResponderWorker>()
            .with_worker::<MilestoneResponderWorker>()
            .with_worker::<MessageRequesterWorker>()
            .with_worker::<MilestoneRequesterWorker>()
            .with_worker_cfg::<MilestoneValidatorWorker>(config.clone())
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
        let network = node.resource::<Network>(); // TODO: Use a weak handle?

        node.resource::<Bus>()
            .add_listener::<(), _, _>(move |latest_milestone: &LatestMilestoneChanged| {
                info!(
                    "New milestone {} {}.",
                    *latest_milestone.0.index, latest_milestone.0.message_id
                );
                tangle.upgrade().map(|tangle| {
                    tangle.update_latest_milestone_index(latest_milestone.0.index);

                    Protocol::broadcast_heartbeat(
                        &network,
                        tangle.get_latest_solid_milestone_index(),
                        tangle.get_pruning_index(),
                        latest_milestone.0.index,
                    );
                });
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
        let network = node.resource::<Network>(); // TODO: Use a weak handle?
        let requested_milestones = node.resource::<RequestedMilestones>();

        node.resource::<Bus>()
            .add_listener::<(), _, _>(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
                tangle.upgrade().map(|tangle| {
                    debug!("New solid milestone {}.", *latest_solid_milestone.0.index);
                    tangle.update_latest_solid_milestone_index(latest_solid_milestone.0.index);

                    let ms_sync_count = config.workers.ms_sync_count;
                    let next_ms = latest_solid_milestone.0.index + MilestoneIndex(ms_sync_count);

                    if tangle.contains_milestone(next_ms) {
                        if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(next_ms)) {
                            error!("Sending solidification event failed: {}", e);
                        }
                    } else {
                        Protocol::request_milestone(
                            &tangle,
                            &milestone_requester,
                            &*requested_milestones,
                            next_ms,
                            None,
                        );
                    }

                    Protocol::broadcast_heartbeat(
                        &network,
                        latest_solid_milestone.0.index,
                        tangle.get_pruning_index(),
                        tangle.get_latest_milestone_index(),
                    );
                });
            });
    }

    pub(crate) fn get() -> &'static Protocol {
        *PROTOCOL.read().as_ref().expect("Uninitialized protocol.")
    }

    pub async fn register<N: Node>(
        node: &N,
        id: PeerId,
        address: Multiaddr,
    ) -> (flume::Sender<Vec<u8>>, oneshot::Sender<()>) {
        // TODO check if not already added ?

        let peer = Arc::new(Peer::new(id, address));

        let (receiver_tx, receiver_rx) = flume::unbounded();
        let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

        Protocol::get().peer_manager.add(peer.clone()).await;

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();

        spawn(
            PeerWorker::new(
                peer,
                node.worker::<HasherWorker>().unwrap().tx.clone(),
                node.worker::<MessageResponderWorker>().unwrap().tx.clone(),
                node.worker::<MilestoneResponderWorker>().unwrap().tx.clone(),
                node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone(),
            )
            .run(tangle.clone(), requested_milestones, receiver_rx, receiver_shutdown_rx),
        );

        (receiver_tx, receiver_shutdown_tx)
    }
}
