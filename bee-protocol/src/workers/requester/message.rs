// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::MessageRequestPacket, peer::PeerManager, sender::Sender, storage::StorageBackend, MetricsWorker,
        PeerManagerResWorker,
    },
};

use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_runtime::{node::Node, resource::ResourceHandle, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};

use async_priority_queue::PriorityQueue;
use async_trait::async_trait;
use backstage::core::{Actor, ActorError, ActorResult, IntervalChannel, Rt, SupHandle};
use futures::StreamExt;
use fxhash::FxBuildHasher;
use log::{debug, info, trace};
use tokio::sync::RwLock;

use std::{
    any::TypeId,
    cmp::{Ord, Ordering, PartialOrd},
    collections::HashMap,
    convert::Infallible,
    marker::PhantomData,
    sync::Arc,
    time::Instant,
};

const RETRY_INTERVAL_MS: u64 = 2500;

pub async fn request_message<B: StorageBackend>(
    tangle: &Tangle<B>,
    message_requester: &MessageRequesterWorker,
    requested_messages: &RequestedMessages,
    message_id: MessageId,
    index: MilestoneIndex,
) {
    if !tangle.contains(&message_id).await
        && !tangle.is_solid_entry_point(&message_id).await
        && !requested_messages.contains(&message_id).await
    {
        message_requester.request(MessageRequesterWorkerEvent(message_id, index));
    }
}

#[derive(Default)]
pub struct RequestedMessages(RwLock<HashMap<MessageId, (MilestoneIndex, Instant), FxBuildHasher>>);

#[allow(clippy::len_without_is_empty)]
impl RequestedMessages {
    pub async fn contains(&self, message_id: &MessageId) -> bool {
        self.0.read().await.contains_key(message_id)
    }

    pub(crate) async fn insert(&self, message_id: MessageId, index: MilestoneIndex) {
        let now = Instant::now();
        self.0.write().await.insert(message_id, (index, now));
    }

    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.0.read().await.is_empty()
    }

    pub(crate) async fn remove(&self, message_id: &MessageId) -> Option<(MilestoneIndex, Instant)> {
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
    let guard = peer_manager.0.read().await;

    for _ in 0..guard.keys.len() {
        let peer_id = &guard.keys[*counter % guard.keys.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id).await {
            if (*peer).0.has_data(index) {
                Sender::<MessageRequestPacket>::send(
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequestPacket::new(message_id),
                )
                .await;
                return;
            }
        }
    }

    for _ in 0..guard.keys.len() {
        let peer_id = &guard.keys[*counter % guard.keys.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id).await {
            if (*peer).0.maybe_has_data(index) {
                Sender::<MessageRequestPacket>::send(
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequestPacket::new(message_id),
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
    tangle: &Tangle<B>,
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

pub struct MessageRetryerActor<B: StorageBackend> {
    _marker: PhantomData<B>,
}

impl<B: StorageBackend> Default for MessageRetryerActor<B> {
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

#[async_trait]
impl<S, B: StorageBackend> Actor<S> for MessageRetryerActor<B>
where
    S: SupHandle<Self>,
{
    type Data = (
        ResourceHandle<Tangle<B>>,
        ResourceHandle<RequestedMessages>,
        ResourceHandle<PeerManager>,
        ResourceHandle<NodeMetrics>,
    );
    type Channel = IntervalChannel<RETRY_INTERVAL_MS>;

    async fn init(&mut self, rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        // This should be the ID of the supervisor.
        let parent_id = rt
            .parent_id()
            .ok_or_else(|| ActorError::aborted_msg("gossip actor has no parent"))?;

        // The event bus should be under the supervisor's ID.
        let tangle = rt
            .lookup::<ResourceHandle<Tangle<B>>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("tangle is not available"))?;

        // The requested messages should be under the supervisor's ID.
        let requested_messages = rt
            .lookup::<ResourceHandle<RequestedMessages>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("requested messages is not available"))?;

        // The peer manager should be under the supervisor's ID.
        let peer_manager = rt
            .lookup::<ResourceHandle<PeerManager>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("peer manager is not available"))?;

        // The node metrics should be under the supervisor's ID.
        let node_metrics = rt
            .lookup::<ResourceHandle<NodeMetrics>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("node metrics is not available"))?;

        Ok((tangle, requested_messages, peer_manager, node_metrics))
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, data: Self::Data) -> ActorResult<()> {
        info!("Running.");

        let (tangle, requested_messages, peer_manager, metrics) = data;

        let mut counter: usize = 0;

        while rt.inbox_mut().next().await.is_some() {
            retry_requests(&requested_messages, &peer_manager, &metrics, &tangle, &mut counter).await;
        }

        info!("Stopped.");

        Ok(())
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

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Retryer running.");

            shutdown.await.unwrap();

            info!("Retryer stopped.");
        });

        Ok(Self { req_queue })
    }
}
