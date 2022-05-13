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
    types::metrics::NodeMetrics,
    workers::{
        packets::MessageRequestPacket, peer::PeerManager, sender::Sender, storage::StorageBackend, MetricsWorker,
        PeerManagerResWorker,
    },
};

const RETRY_INTERVAL: Duration = Duration::from_millis(2500);

pub async fn request_message<B: StorageBackend>(
    tangle: &Tangle<B>,
    message_requester: &MessageRequesterWorker,
    requested_messages: &RequestedMessages,
    message_id: BlockId,
    index: MilestoneIndex,
) {
    if !tangle.contains(&message_id)
        && !tangle.is_solid_entry_point(&message_id).await
        && !requested_messages.contains(&message_id)
    {
        message_requester.request(MessageRequesterWorkerEvent(message_id, index));
    }
}

#[derive(Default)]
pub struct RequestedMessages(RwLock<HashMap<BlockId, (MilestoneIndex, Instant), FxBuildHasher>>);

#[allow(clippy::len_without_is_empty)]
impl RequestedMessages {
    pub fn contains(&self, message_id: &BlockId) -> bool {
        self.0.read().contains_key(message_id)
    }

    pub(crate) fn insert(&self, message_id: BlockId, index: MilestoneIndex) {
        let now = Instant::now();
        self.0.write().insert(message_id, (index, now));
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    pub(crate) fn remove(&self, message_id: &BlockId) -> Option<(MilestoneIndex, Instant)> {
        self.0.write().remove(message_id)
    }
}

#[derive(Eq, PartialEq)]
pub struct MessageRequesterWorkerEvent(pub(crate) BlockId, pub(crate) MilestoneIndex);

impl Ord for MessageRequesterWorkerEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1).reverse()
    }
}

impl PartialOrd for MessageRequesterWorkerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
pub struct MessageRequesterWorker {
    req_queue: Arc<PriorityQueue<MessageRequesterWorkerEvent>>,
}

impl MessageRequesterWorker {
    pub fn request(&self, request: MessageRequesterWorkerEvent) {
        self.req_queue.push(request);
    }
}

fn process_request(
    message_id: BlockId,
    index: MilestoneIndex,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    requested_messages: &RequestedMessages,
) {
    if requested_messages.contains(&message_id) {
        return;
    }

    if peer_manager.is_empty() {
        return;
    }

    requested_messages.insert(message_id, index);

    process_request_unchecked(message_id, index, peer_manager, metrics);
}

fn process_request_unchecked(
    message_id: BlockId,
    index: MilestoneIndex,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
) {
    let message_request = MessageRequestPacket::new(message_id);

    if let Some(peer_id) = peer_manager
        .fair_find(|peer| peer.has_data(index))
        .or_else(|| peer_manager.fair_find(|peer| peer.maybe_has_data(index)))
    {
        Sender::<MessageRequestPacket>::send(&message_request, &peer_id, peer_manager, metrics)
    }
}

fn retry_requests<B: StorageBackend>(
    requested_messages: &RequestedMessages,
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
    for (message_id, (index, instant)) in requested_messages.0.read().iter() {
        if now
            .checked_duration_since(*instant)
            .map_or(false, |d| d > RETRY_INTERVAL)
        {
            to_retry.push((*message_id, *index));
            retry_counts += 1;
        }
    }

    for (message_id, index) in to_retry {
        if tangle.contains(&message_id) {
            requested_messages.remove(&message_id);
        } else {
            process_request_unchecked(message_id, index, peer_manager, metrics);
        }
    }

    if retry_counts > 0 {
        debug!("Retried {} messages.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MessageRequesterWorker
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

        let requested_messages: RequestedMessages = Default::default();
        node.register_resource(requested_messages);

        let requested_messages = node.resource::<RequestedMessages>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>({
            let req_queue = req_queue.clone();
            |shutdown| async move {
                info!("Requester running.");

                let mut receiver = ShutdownStream::new(shutdown, req_queue.incoming());

                while let Some(MessageRequesterWorkerEvent(message_id, index)) = receiver.next().await {
                    trace!("Requesting message {}.", message_id);

                    process_request(message_id, index, &peer_manager, &metrics, &requested_messages);
                }

                info!("Requester stopped.");
            }
        });

        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Retryer running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(RETRY_INTERVAL)));

            while ticker.next().await.is_some() {
                retry_requests(&requested_messages, &peer_manager, &metrics, &tangle);
            }

            info!("Retryer stopped.");
        });

        Ok(Self { req_queue })
    }
}
