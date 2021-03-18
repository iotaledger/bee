// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{metrics::NodeMetrics, packets::MilestoneRequest},
    workers::{peer::PeerManager, sender::Sender, storage::StorageBackend, MetricsWorker, PeerManagerResWorker},
};

use bee_message::milestone::MilestoneIndex;
use bee_network::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::StreamExt;
use fxhash::FxBuildHasher;
use log::{debug, info};
use tokio::{
    sync::{mpsc, RwLock},
    time::interval,
};
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};

use std::{
    any::TypeId,
    collections::HashMap,
    convert::Infallible,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_MS: u64 = 2500;

// TODO pub ?
#[derive(Default)]
pub struct RequestedMilestones(RwLock<HashMap<MilestoneIndex, Instant, FxBuildHasher>>);

impl RequestedMilestones {
    pub async fn contains(&self, index: &MilestoneIndex) -> bool {
        self.0.read().await.contains_key(index)
    }

    pub async fn insert(&self, index: MilestoneIndex) {
        let now = Instant::now();
        self.0.write().await.insert(index, now);
    }

    // pub async fn len(&self) -> usize {
    //     self.0.read().await.len()
    // }

    pub async fn remove(&self, index: &MilestoneIndex) -> Option<Instant> {
        self.0.write().await.remove(index)
    }
}

pub(crate) struct MilestoneRequesterWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Option<PeerId>);

pub(crate) struct MilestoneRequesterWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
}

async fn process_request(
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    requested_milestones: &RequestedMilestones,
    counter: &mut usize,
) {
    if requested_milestones.contains(&index).await {
        return;
    }

    if peer_manager.is_empty().await {
        return;
    }

    if index.0 != 0 {
        requested_milestones.insert(index).await;
    }

    process_request_unchecked(index, peer_id, peer_manager, metrics, counter).await;
}

async fn process_request_unchecked(
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    counter: &mut usize,
) {
    match peer_id {
        Some(peer_id) => {
            Sender::<MilestoneRequest>::send(peer_manager, metrics, &peer_id, MilestoneRequest::new(*index)).await;
        }
        None => {
            let guard = peer_manager.peers_keys.read().await;

            for _ in 0..guard.len() {
                let peer_id = &guard[*counter % guard.len()];

                *counter += 1;

                if let Some(peer) = peer_manager.get(peer_id).await {
                    // TODO also request if has_data ?
                    if (*peer).0.maybe_has_data(index) {
                        Sender::<MilestoneRequest>::send(
                            peer_manager,
                            metrics,
                            &peer_id,
                            MilestoneRequest::new(*index),
                        )
                        .await;
                        return;
                    }
                }
            }
        }
    }
}

async fn retry_requests<B: StorageBackend>(
    requested_milestones: &RequestedMilestones,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    tangle: &MsTangle<B>,
    counter: &mut usize,
) {
    if peer_manager.is_empty().await {
        return;
    }

    let now = Instant::now();
    let mut retry_counts: usize = 0;
    let mut to_retry = Vec::with_capacity(1024);

    // TODO this needs abstraction
    for (index, instant) in requested_milestones.0.read().await.iter() {
        if now
            .checked_duration_since(*instant)
            .map_or(false, |d| d.as_millis() as u64 > RETRY_INTERVAL_MS)
        {
            to_retry.push(*index);
            retry_counts += 1;
        };
    }

    for index in to_retry {
        if tangle.contains_milestone(index).await {
            requested_milestones.remove(&index).await;
        } else {
            process_request_unchecked(index, None, peer_manager, metrics, counter).await;
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

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Requester running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));
            let mut counter: usize = 0;

            while let Some(MilestoneRequesterWorkerEvent(index, peer_id)) = receiver.next().await {
                if !tangle.contains_milestone(index).await {
                    debug!("Requesting milestone {}.", *index);
                    process_request(
                        index,
                        peer_id,
                        &peer_manager,
                        &metrics,
                        &requested_milestones,
                        &mut counter,
                    )
                    .await;
                }
            }

            info!("Requester stopped.");
        });

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Retryer running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_millis(RETRY_INTERVAL_MS))),
            );
            let mut counter: usize = 0;

            while ticker.next().await.is_some() {
                retry_requests(&requested_milestones, &peer_manager, &metrics, &tangle, &mut counter).await;
            }

            info!("Retryer stopped.");
        });

        Ok(Self { tx })
    }
}
