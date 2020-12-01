// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    packet::MilestoneRequest,
    protocol::{Protocol, Sender},
    tangle::MsTangle,
    worker::TangleWorker,
};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_network::{Network, PeerId};

use async_trait::async_trait;
use dashmap::DashMap;
use futures::{select, StreamExt};
use log::{debug, info};
use tokio::time::interval;

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
    pub(crate) tx: flume::Sender<MilestoneRequesterWorkerEvent>,
}

async fn process_request(
    network: &Network,
    requested_milestones: &RequestedMilestones,
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    counter: &mut usize,
) {
    if requested_milestones.contains_key(&index) {
        return;
    }

    process_request_unchecked(network, index, peer_id, counter).await;

    if index.0 != 0 {
        requested_milestones.insert(index, Instant::now());
    }
}

/// Return `true` if the milestone was requested
async fn process_request_unchecked(
    network: &Network,
    index: MilestoneIndex,
    peer_id: Option<PeerId>,
    counter: &mut usize,
) -> bool {
    if Protocol::get().peer_manager.peers.is_empty() {
        return false;
    }

    match peer_id {
        Some(peer_id) => {
            Sender::<MilestoneRequest>::send(network, &peer_id, MilestoneRequest::new(*index));
            true
        }
        None => {
            let guard = Protocol::get().peer_manager.peers_keys.read().await;

            for _ in 0..guard.len() {
                let peer_id = &guard[*counter % guard.len()];

                *counter += 1;

                if let Some(peer) = Protocol::get().peer_manager.peers.get(peer_id) {
                    if peer.maybe_has_data(index) {
                        Sender::<MilestoneRequest>::send(network, &peer_id, MilestoneRequest::new(*index));
                        return true;
                    }
                }
            }

            false
        }
    }
}

async fn retry_requests(network: &Network, requested_milestones: &RequestedMilestones, counter: &mut usize) {
    let mut retry_counts: usize = 0;

    for mut milestone in requested_milestones.iter_mut() {
        let (index, instant) = milestone.pair_mut();
        let now = Instant::now();
        if (now - *instant).as_secs() > RETRY_INTERVAL_SEC
            && process_request_unchecked(network, *index, None, counter).await
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
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let network = node.resource::<Network>();
        let requested_milestones: RequestedMilestones = Default::default();
        node.register_resource(requested_milestones);
        let requested_milestones = node.resource::<RequestedMilestones>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut counter: usize = 0;
            let mut timeouts = interval(Duration::from_secs(RETRY_INTERVAL_SEC)).fuse();

            loop {
                select! {
                    _ = timeouts.next() => retry_requests(&network, &*requested_milestones, &mut counter).await,
                    entry = receiver.next() => match entry {
                        Some(MilestoneRequesterWorkerEvent(index, peer_id)) => {
                            if !tangle.contains_milestone(index.into()) {
                                debug!("Requesting milestone {}.", *index);
                                process_request(&network, &*requested_milestones, index, peer_id, &mut counter).await;
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
