// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::TypeId,
    cmp::{Ord, Ordering, PartialOrd},
    collections::HashMap,
    convert::Infallible,
    sync::Arc,
    time::{Duration, Instant},
};

use async_priority_queue::PriorityQueue;
use async_trait::async_trait;
use bee_block::{payload::milestone::MilestoneIndex, BlockId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};
use futures::StreamExt;
use fxhash::FxBuildHasher;
use log::{debug, info, trace};
use parking_lot::RwLock;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use crate::{
    packets::BlockRequestPacket, peer::PeerManager, sender::Sender, storage::StorageBackend,
    types::metrics::NodeMetrics, MetricsWorker, PeerManagerResWorker,
};

const RETRY_INTERVAL: Duration = Duration::from_millis(2500);

pub async fn request_block<B: StorageBackend>(
    tangle: &Tangle<B>,
    block_requester: &BlockRequesterWorker,
    requested_blocks: &RequestedBlocks,
    block_id: BlockId,
    index: MilestoneIndex,
) {
    if !tangle.contains(&block_id)
        && !tangle.is_solid_entry_point(&block_id).await
        && !requested_blocks.contains(&block_id)
    {
        block_requester.request(BlockRequesterWorkerEvent(block_id, index));
    }
}

#[derive(Default)]
pub struct RequestedBlocks(RwLock<HashMap<BlockId, (MilestoneIndex, Instant), FxBuildHasher>>);

#[allow(clippy::len_without_is_empty)]
impl RequestedBlocks {
    pub fn contains(&self, block_id: &BlockId) -> bool {
        self.0.read().contains_key(block_id)
    }

    pub(crate) fn insert(&self, block_id: BlockId, index: MilestoneIndex) {
        let now = Instant::now();
        self.0.write().insert(block_id, (index, now));
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    pub(crate) fn remove(&self, block_id: &BlockId) -> Option<(MilestoneIndex, Instant)> {
        self.0.write().remove(block_id)
    }
}

#[derive(Eq, PartialEq)]
pub struct BlockRequesterWorkerEvent(pub(crate) BlockId, pub(crate) MilestoneIndex);

impl Ord for BlockRequesterWorkerEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1).reverse()
    }
}

impl PartialOrd for BlockRequesterWorkerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
pub struct BlockRequesterWorker {
    req_queue: Arc<PriorityQueue<BlockRequesterWorkerEvent>>,
}

impl BlockRequesterWorker {
    pub fn request(&self, request: BlockRequesterWorkerEvent) {
        self.req_queue.push(request);
    }
}

fn process_request(
    block_id: BlockId,
    index: MilestoneIndex,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    requested_blocks: &RequestedBlocks,
) {
    if requested_blocks.contains(&block_id) {
        return;
    }

    if peer_manager.is_empty() {
        return;
    }

    requested_blocks.insert(block_id, index);

    process_request_unchecked(block_id, index, peer_manager, metrics);
}

fn process_request_unchecked(
    block_id: BlockId,
    index: MilestoneIndex,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
) {
    let block_request = BlockRequestPacket::new(block_id);

    if let Some(peer_id) = peer_manager
        .fair_find(|peer| peer.has_data(index))
        .or_else(|| peer_manager.fair_find(|peer| peer.maybe_has_data(index)))
    {
        Sender::<BlockRequestPacket>::send(&block_request, &peer_id, peer_manager, metrics)
    }
}

fn retry_requests<B: StorageBackend>(
    requested_blocks: &RequestedBlocks,
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
    for (block_id, (index, instant)) in requested_blocks.0.read().iter() {
        if now
            .checked_duration_since(*instant)
            .map_or(false, |d| d > RETRY_INTERVAL)
        {
            to_retry.push((*block_id, *index));
            retry_counts += 1;
        }
    }

    for (block_id, index) in to_retry {
        if tangle.contains(&block_id) {
            requested_blocks.remove(&block_id);
        } else {
            process_request_unchecked(block_id, index, peer_manager, metrics);
        }
    }

    if retry_counts > 0 {
        debug!("Retried {} blocks.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for BlockRequesterWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<TangleWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let req_queue = Arc::new(PriorityQueue::new());

        let requested_blocks: RequestedBlocks = Default::default();
        node.register_resource(requested_blocks);

        let requested_blocks = node.resource::<RequestedBlocks>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>({
            let req_queue = req_queue.clone();
            |shutdown| async move {
                info!("Requester running.");

                let mut receiver = ShutdownStream::new(shutdown, req_queue.incoming());

                while let Some(BlockRequesterWorkerEvent(block_id, index)) = receiver.next().await {
                    trace!("Requesting block {}.", block_id);

                    process_request(block_id, index, &peer_manager, &metrics, &requested_blocks);
                }

                info!("Requester stopped.");
            }
        });

        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_blocks = node.resource::<RequestedBlocks>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Retryer running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(RETRY_INTERVAL)));

            while ticker.next().await.is_some() {
                retry_requests(&requested_blocks, &peer_manager, &metrics, &tangle);
            }

            info!("Retryer stopped.");
        });

        Ok(Self { req_queue })
    }
}
