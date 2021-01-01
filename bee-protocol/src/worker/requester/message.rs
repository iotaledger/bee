// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    packet::MessageRequest,
    peer::PeerManager,
    worker::{MetricsWorker, PeerManagerWorker},
    ProtocolMetrics, Sender,
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{
    node::{Node, ResHandle},
    worker::Worker,
};
use bee_message::MessageId;
use bee_network::NetworkController;

use async_trait::async_trait;
use dashmap::DashMap;
use futures::StreamExt;
use log::{debug, info, trace};
use tokio::{sync::mpsc, time::interval};

use std::{
    any::TypeId,
    convert::Infallible,
    ops::Deref,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_SEC: u64 = 5;

#[derive(Default)]
pub(crate) struct RequestedMessages(DashMap<MessageId, (MilestoneIndex, Instant)>);

impl Deref for RequestedMessages {
    type Target = DashMap<MessageId, (MilestoneIndex, Instant)>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    peer_manager: &ResHandle<PeerManager>,
    metrics: &ResHandle<ProtocolMetrics>,
    requested_messages: &ResHandle<RequestedMessages>,
    counter: &mut usize,
) {
    if requested_messages.contains_key(&message_id) {
        return;
    }

    if process_request_unchecked(message_id, index, network, peer_manager, metrics, counter).await {
        requested_messages.insert(message_id, (index, Instant::now()));
    }
}

/// Return `true` if the message was requested.
async fn process_request_unchecked(
    message_id: MessageId,
    index: MilestoneIndex,
    network: &NetworkController,
    peer_manager: &ResHandle<PeerManager>,
    metrics: &ResHandle<ProtocolMetrics>,
    counter: &mut usize,
) -> bool {
    if peer_manager.is_empty() {
        return false;
    }

    let guard = peer_manager.peers_keys.read().await;

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id) {
            if peer.has_data(index) {
                Sender::<MessageRequest>::send(
                    network,
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequest::new(message_id.as_ref()),
                );
                return true;
            }
        }
    }

    let mut requested = false;

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.get(peer_id) {
            if peer.maybe_has_data(index) {
                Sender::<MessageRequest>::send(
                    network,
                    peer_manager,
                    metrics,
                    peer_id,
                    MessageRequest::new(message_id.as_ref()),
                );
                requested = true;
            }
        }
    }

    requested
}

async fn retry_requests(
    network: &NetworkController,
    requested_messages: &ResHandle<RequestedMessages>,
    peer_manager: &ResHandle<PeerManager>,
    metrics: &ResHandle<ProtocolMetrics>,
    counter: &mut usize,
) {
    let mut retry_counts: usize = 0;

    for mut message in requested_messages.iter_mut() {
        let (message_id, (index, instant)) = message.pair_mut();
        let now = Instant::now();
        if (now - *instant).as_secs() > RETRY_INTERVAL_SEC
            && process_request_unchecked(*message_id, *index, network, peer_manager, metrics, counter).await
        {
            // TODO should we actually update ?
            *instant = now;
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
        vec![TypeId::of::<PeerManagerWorker>(), TypeId::of::<MetricsWorker>()].leak()
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

            let mut receiver = ShutdownStream::new(shutdown, rx);
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

            let mut ticker = ShutdownStream::new(shutdown, interval(Duration::from_secs(RETRY_INTERVAL_SEC)));
            let mut counter: usize = 0;

            while ticker.next().await.is_some() {
                retry_requests(&network, &requested_messages, &peer_manager, &metrics, &mut counter).await;
            }

            info!("Retryer stopped.");
        });

        Ok(Self { tx })
    }
}
