// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    packet::MilestoneRequest,
    peer::PeerManager,
    tangle::MsTangle,
    worker::{MetricsWorker, PeerManagerWorker, TangleWorker},
    ProtocolMetrics, Sender,
};

use bee_common::{
    node::{Node, ResHandle},
    shutdown_stream::ShutdownStream,
    worker::Worker,
};
use bee_network::{NetworkController, PeerId};

use async_trait::async_trait;
use dashmap::DashMap;
use futures::{select, StreamExt};
use log::{debug, info};
use tokio::{sync::mpsc, time::interval};

use std::{
    any::TypeId,
    convert::Infallible,
    ops::Deref,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_SEC: u64 = 5;

// TODO pub ?
#[derive(Default)]
pub struct RequestedMilestones(DashMap<MilestoneIndex, Instant>);

impl Deref for RequestedMilestones {
    type Target = DashMap<MilestoneIndex, Instant>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct MilestoneRequesterWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Option<PeerId>);

pub(crate) struct MilestoneRequesterWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
}

async fn process_request(
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    network: &NetworkController,
    peer_manager: &ResHandle<PeerManager>,
    metrics: &ResHandle<ProtocolMetrics>,
    requested_milestones: &ResHandle<RequestedMilestones>,
    counter: &mut usize,
) {
    if requested_milestones.contains_key(&index) {
        return;
    }

    process_request_unchecked(index, peer_id, network, peer_manager, metrics, counter).await;

    if index.0 != 0 {
        requested_milestones.insert(index, Instant::now());
    }
}

/// Return `true` if the milestone was requested
async fn process_request_unchecked(
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    network: &NetworkController,
    peer_manager: &ResHandle<PeerManager>,
    metrics: &ResHandle<ProtocolMetrics>,
    counter: &mut usize,
) -> bool {
    if peer_manager.peers.is_empty() {
        return false;
    }

    match peer_id {
        Some(peer_id) => {
            Sender::<MilestoneRequest>::send(network, metrics, &peer_id, MilestoneRequest::new(*index));
            true
        }
        None => {
            let guard = peer_manager.peers_keys.read().await;

            for _ in 0..guard.len() {
                let peer_id = &guard[*counter % guard.len()];

                *counter += 1;

                if let Some(peer) = peer_manager.peers.get(peer_id) {
                    if peer.maybe_has_data(index) {
                        Sender::<MilestoneRequest>::send(network, metrics, &peer_id, MilestoneRequest::new(*index));
                        return true;
                    }
                }
            }

            false
        }
    }
}

async fn retry_requests(
    network: &NetworkController,
    peer_manager: &ResHandle<PeerManager>,
    metrics: &ResHandle<ProtocolMetrics>,
    requested_milestones: &ResHandle<RequestedMilestones>,
    counter: &mut usize,
) {
    let mut retry_counts: usize = 0;

    for mut milestone in requested_milestones.iter_mut() {
        let (index, instant) = milestone.pair_mut();
        let now = Instant::now();
        if (now - *instant).as_secs() > RETRY_INTERVAL_SEC
            && process_request_unchecked(*index, None, network, peer_manager, metrics, counter).await
        {
            *instant = now;
            retry_counts += 1;
        };
    }

    if retry_counts > 0 {
        debug!("Retried {} milestones.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneRequesterWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<PeerManagerWorker>(),
            TypeId::of::<MetricsWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let requested_milestones: RequestedMilestones = Default::default();
        node.register_resource(requested_milestones);

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let network = node.resource::<NetworkController>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            let mut counter: usize = 0;
            let mut timeouts = interval(Duration::from_secs(RETRY_INTERVAL_SEC)).fuse();

            loop {
                select! {
                    _ = timeouts.next() => retry_requests(&network, &peer_manager, &metrics, &requested_milestones, &mut counter).await,
                    entry = receiver.next() => match entry {
                        Some(MilestoneRequesterWorkerEvent(index, peer_id)) => {
                            if !tangle.contains_milestone(index.into()) {
                                debug!("Requesting milestone {}.", *index);
                                process_request(index, peer_id, &network, &peer_manager, &metrics, &requested_milestones, &mut counter).await;
                            }
                        },
                        None => break,
                    },
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }

    // TODO stop + remove_resource
}
