// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::TypeId,
    collections::HashMap,
    convert::Infallible,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use bee_gossip::PeerId;
use bee_message::payload::milestone::MilestoneIndex;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};
use futures::StreamExt;
use fxhash::FxBuildHasher;
use log::{debug, info, warn};
use parking_lot::RwLock;
use tokio::{sync::mpsc, time::interval};
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::MilestoneRequestPacket, peer::PeerManager, sender::Sender, storage::StorageBackend, MetricsWorker,
        PeerManagerResWorker,
    },
};

const RETRY_INTERVAL: Duration = Duration::from_millis(2500);

pub(crate) fn request_milestone<B: StorageBackend>(
    tangle: &Tangle<B>,
    milestone_requester: &mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
    requested_milestones: &RequestedMilestones,
    index: MilestoneIndex,
    to: Option<PeerId>,
) {
    if !requested_milestones.contains(&index) && !tangle.contains_milestone_metadata(index) {
        if let Err(e) = milestone_requester.send(MilestoneRequesterWorkerEvent(index, to)) {
            warn!("Requesting milestone failed: {}.", e);
        }
    }
}

pub(crate) fn request_latest_milestone<B: StorageBackend>(
    tangle: &Tangle<B>,
    milestone_requester: &mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
    requested_milestones: &RequestedMilestones,
    to: Option<PeerId>,
) {
    request_milestone(tangle, milestone_requester, requested_milestones, MilestoneIndex(0), to)
}

#[derive(Default)]
pub struct RequestedMilestones(RwLock<HashMap<MilestoneIndex, Instant, FxBuildHasher>>);

#[allow(clippy::len_without_is_empty)]
impl RequestedMilestones {
    pub fn contains(&self, index: &MilestoneIndex) -> bool {
        self.0.read().contains_key(index)
    }

    pub(crate) fn insert(&self, index: MilestoneIndex) {
        let now = Instant::now();
        self.0.write().insert(index, now);
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    pub(crate) fn remove(&self, index: &MilestoneIndex) -> Option<Instant> {
        self.0.write().remove(index)
    }
}

pub(crate) struct MilestoneRequesterWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Option<PeerId>);

pub(crate) struct MilestoneRequesterWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
}

fn process_request(
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    requested_milestones: &RequestedMilestones,
) {
    if requested_milestones.contains(&index) {
        return;
    }

    if peer_manager.is_empty() {
        return;
    }

    if index.0 != 0 {
        requested_milestones.insert(index);
    }

    process_request_unchecked(index, peer_id, peer_manager, metrics);
}

fn process_request_unchecked(
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
) {
    let milestone_request = MilestoneRequestPacket::new(*index);

    match peer_id {
        Some(peer_id) => {
            Sender::<MilestoneRequestPacket>::send(&milestone_request, &peer_id, peer_manager, metrics);
        }
        None => {
            if let Some(peer_id) = peer_manager.fair_find(|peer| peer.maybe_has_data(index)) {
                Sender::<MilestoneRequestPacket>::send(&milestone_request, &peer_id, peer_manager, metrics);
            }
        }
    }
}

fn retry_requests<B: StorageBackend>(
    requested_milestones: &RequestedMilestones,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    tangle: &Tangle<B>,
) {
    if peer_manager.is_empty() {
        return;
    }

    let now = Instant::now();
    let mut retry_counts: usize = 0;
    let mut to_retry = Vec::with_capacity(1024);

    // TODO this needs abstraction
    for (index, instant) in requested_milestones.0.read().iter() {
        if now
            .checked_duration_since(*instant)
            .map_or(false, |d| d > RETRY_INTERVAL)
        {
            to_retry.push(*index);
            retry_counts += 1;
        };
    }

    for index in to_retry {
        if tangle.contains_milestone_metadata(index) {
            requested_milestones.remove(&index);
        } else {
            process_request_unchecked(index, None, peer_manager, metrics);
        }
    }

    if retry_counts > 0 {
        debug!("Retried {} milestones.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneRequesterWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<MetricsWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let requested_milestones: RequestedMilestones = Default::default();
        node.register_resource(requested_milestones);

        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Requester running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(MilestoneRequesterWorkerEvent(index, peer_id)) = receiver.next().await {
                if !tangle.contains_milestone_metadata(index) {
                    debug!("Requesting milestone {}.", *index);
                    process_request(index, peer_id, &peer_manager, &metrics, &requested_milestones);
                }
            }

            info!("Requester stopped.");
        });

        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Retryer running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(RETRY_INTERVAL)));

            while ticker.next().await.is_some() {
                retry_requests(&requested_milestones, &peer_manager, &metrics, &tangle);
            }

            info!("Retryer stopped.");
        });

        Ok(Self { tx })
    }
}
