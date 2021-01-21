// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::MessageRequest,
    peer::PeerManager,
    worker::{MetricsWorker, PeerManagerResWorker},
    ProtocolMetrics, Sender,
};

use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_network::NetworkController;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::{debug, info, trace};
use tokio::{sync::{RwLock, mpsc}, time::interval};
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};

use std::{
    any::TypeId,
    collections::HashMap,
    convert::Infallible,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_MS: u64 = 2500;

#[derive(Default)]
pub(crate) struct RequestedMessages(RwLock<HashMap<MessageId, (MilestoneIndex, Instant)>>);

impl RequestedMessages {
    pub async fn contains(&self, message_id: &MessageId) -> bool {
        self.0.read().await.contains_key(message_id)
    }

    pub async fn insert(&self, message_id: MessageId, index: MilestoneIndex) {
        self.0.write().await.insert(message_id, (index, Instant::now()));
    }

    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    pub async fn remove(&self, message_id: &MessageId) -> Option<(MilestoneIndex, Instant)> {
        self.0.write().await.remove(message_id)
    }
}

pub(crate) struct MessageRequesterWorkerEvent(pub(crate) MessageId, pub(crate) MilestoneIndex);

pub(crate) struct MessageRequesterWorker {
    pub(crate) tx: mpsc::UnboundedSender<MessageRequesterWorkerEvent>,
}

async fn process_request(
    message_id: MessageId,
    index: MilestoneIndex,
    network: &NetworkController,
    peer_manager: &PeerManager,
    metrics: &ProtocolMetrics,
    requested_messages: &RequestedMessages,
    counter: &mut usize,
) {
    if requested_messages.contains(&message_id).await {
        return;
    }

    if peer_manager.is_empty().await {
        return;
    }

    if process_request_unchecked(message_id, index, network, peer_manager, metrics, counter).await {
        requested_messages.insert(message_id, index).await;
    }
}

/// Return `true` if the message was requested.
async fn process_request_unchecked(
    message_id: MessageId,
    index: MilestoneIndex,
    network: &NetworkController,
    peer_manager: &PeerManager,
    metrics: &ProtocolMetrics,
    counter: &mut usize,
) -> bool {
    let guard = peer_manager.peers_keys.read().await;

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id).await {
            if (*peer).0.has_data(index) {
                Sender::<MessageRequest>::send(
                    network,
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequest::new(message_id.as_ref()),
                )
                .await;
                return true;
            }
        }
    }

    let mut requested = false;

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id).await {
            if (*peer).0.maybe_has_data(index) {
                Sender::<MessageRequest>::send(
                    network,
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequest::new(message_id.as_ref()),
                )
                .await;
                requested = true;
            }
        }
    }

    requested
}

async fn retry_requests(
    network: &NetworkController,
    requested_messages: &RequestedMessages,
    peer_manager: &PeerManager,
    metrics: &ProtocolMetrics,
    counter: &mut usize,
) {
    if peer_manager.is_empty().await {
        return;
    }

    let mut retry_counts: usize = 0;

    // TODO this needs abstraction
    for (message_id, (index, instant)) in requested_messages.0.read().await.iter() {
        if (Instant::now() - *instant).as_millis() as u64 > RETRY_INTERVAL_MS
            && process_request_unchecked(*message_id, *index, network, peer_manager, metrics, counter).await
        {
            retry_counts += 1;
        }
    }

    if retry_counts > 0 {
        debug!("Retried {} messages.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MessageRequesterWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<PeerManagerResWorker>(), TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let requested_messages: RequestedMessages = Default::default();
        node.register_resource(requested_messages);

        let requested_messages = node.resource::<RequestedMessages>();
        let network = node.resource::<NetworkController>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Requester running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));
            let mut counter: usize = 0;

            while let Some(MessageRequesterWorkerEvent(message_id, index)) = receiver.next().await {
                trace!("Requesting message {}.", message_id);

                process_request(
                    message_id,
                    index,
                    &network,
                    &peer_manager,
                    &metrics,
                    &requested_messages,
                    &mut counter,
                )
                .await
            }

            info!("Requester stopped.");
        });

        let requested_messages = node.resource::<RequestedMessages>();
        let network = node.resource::<NetworkController>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Retryer running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_millis(RETRY_INTERVAL_MS))),
            );
            let mut counter: usize = 0;

            while ticker.next().await.is_some() {
                retry_requests(&network, &requested_messages, &peer_manager, &metrics, &mut counter).await;
            }

            info!("Retryer stopped.");
        });

        Ok(Self { tx })
    }
}
