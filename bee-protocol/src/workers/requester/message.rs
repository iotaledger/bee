// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::MessageRequest, peer::PeerManager, sender::Sender, storage::StorageBackend, MetricsWorker,
        PeerManagerResWorker,
    },
};

use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_priority_queue::PriorityQueue;
use async_trait::async_trait;
use futures::StreamExt;
use fxhash::FxBuildHasher;
use log::{debug, info, trace};
use tokio::{sync::RwLock, time::interval};
use tokio_stream::wrappers::IntervalStream;

use std::{
    any::TypeId,
    cmp::{Ord, Ordering, PartialOrd},
    collections::HashMap,
    convert::Infallible,
    sync::Arc,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_MS: u64 = 2500;

#[derive(Default)]
pub struct RequestedMessages(RwLock<HashMap<MessageId, (MilestoneIndex, Instant), FxBuildHasher>>);

impl RequestedMessages {
    pub async fn contains(&self, message_id: &MessageId) -> bool {
        self.0.read().await.contains_key(message_id)
    }

    pub async fn insert(&self, message_id: MessageId, index: MilestoneIndex) {
        let now = Instant::now();
        self.0.write().await.insert(message_id, (index, now));
    }

    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    pub async fn remove(&self, message_id: &MessageId) -> Option<(MilestoneIndex, Instant)> {
        self.0.write().await.remove(message_id)
    }
}

#[derive(Eq, PartialEq)]
pub struct MessageRequesterWorkerEvent(pub(crate) MessageId, pub(crate) MilestoneIndex);

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

async fn process_request(
    message_id: MessageId,
    index: MilestoneIndex,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    requested_messages: &RequestedMessages,
    counter: &mut usize,
) {
    if requested_messages.contains(&message_id).await {
        return;
    }

    if peer_manager.is_empty().await {
        return;
    }

    requested_messages.insert(message_id, index).await;

    process_request_unchecked(message_id, index, peer_manager, metrics, counter).await;
}

async fn process_request_unchecked(
    message_id: MessageId,
    index: MilestoneIndex,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    counter: &mut usize,
) {
    let guard = peer_manager.peers_keys.read().await;

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id).await {
            if (*peer).0.has_data(index) {
                Sender::<MessageRequest>::send(
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequest::new(message_id.as_ref()),
                )
                .await;
                return;
            }
        }
    }

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id).await {
            if (*peer).0.maybe_has_data(index) {
                Sender::<MessageRequest>::send(
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequest::new(message_id.as_ref()),
                )
                .await;
            }
        }
    }
}

async fn retry_requests<B: StorageBackend>(
    requested_messages: &RequestedMessages,
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
    for (message_id, (index, instant)) in requested_messages.0.read().await.iter() {
        if now
            .checked_duration_since(*instant)
            .map_or(false, |d| d.as_millis() as u64 > RETRY_INTERVAL_MS)
        {
            to_retry.push((*message_id, *index));
            retry_counts += 1;
        }
    }

    for (message_id, index) in to_retry {
        if tangle.contains(&message_id).await {
            requested_messages.remove(&message_id).await;
        } else {
            process_request_unchecked(message_id, index, peer_manager, metrics, counter).await;
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
                let mut counter: usize = 0;

                while let Some(MessageRequesterWorkerEvent(message_id, index)) = receiver.next().await {
                    trace!("Requesting message {}.", message_id);

                    process_request(
                        message_id,
                        index,
                        &peer_manager,
                        &metrics,
                        &requested_messages,
                        &mut counter,
                    )
                    .await
                }

                info!("Requester stopped.");
            }
        });

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
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
                retry_requests(&requested_messages, &peer_manager, &metrics, &tangle, &mut counter).await;
            }

            info!("Retryer stopped.");
        });

        Ok(Self { req_queue })
    }
}
