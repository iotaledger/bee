// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, packet::MessageRequest, peer::PeerManager, worker::PeerManagerWorker, Sender};

use bee_common::{
    node::{Node, ResHandle},
    shutdown_stream::ShutdownStream,
    worker::Worker,
};
use bee_message::MessageId;
use bee_network::NetworkController;

use async_trait::async_trait;
use dashmap::DashMap;
use futures::{select, StreamExt};
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
    requested_messages: &RequestedMessages,
    counter: &mut usize,
) {
    if requested_messages.contains_key(&message_id) {
        return;
    }

    if process_request_unchecked(message_id, index, network, peer_manager, counter).await {
        requested_messages.insert(message_id, (index, Instant::now()));
    }
}

/// Return `true` if the message was requested.
async fn process_request_unchecked(
    message_id: MessageId,
    index: MilestoneIndex,
    network: &NetworkController,
    peer_manager: &ResHandle<PeerManager>,
    counter: &mut usize,
) -> bool {
    if peer_manager.peers.is_empty() {
        return false;
    }

    let guard = peer_manager.peers_keys.read().await;

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.peers.get(peer_id) {
            if peer.has_data(index) {
                Sender::<MessageRequest>::send(network, peer_id, MessageRequest::new(message_id.as_ref()));
                return true;
            }
        }
    }

    for _ in 0..guard.len() {
        let peer_id = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = peer_manager.peers.get(peer_id) {
            if peer.maybe_has_data(index) {
                Sender::<MessageRequest>::send(network, peer_id, MessageRequest::new(message_id.as_ref()));
                return true;
            }
        }
    }

    false
}

async fn retry_requests(
    network: &NetworkController,
    requested_messages: &RequestedMessages,
    peer_manager: &ResHandle<PeerManager>,
    counter: &mut usize,
) {
    let mut retry_counts: usize = 0;

    for mut message in requested_messages.iter_mut() {
        let (message_id, (index, instant)) = message.pair_mut();
        let now = Instant::now();
        if (now - *instant).as_secs() > RETRY_INTERVAL_SEC
            && process_request_unchecked(*message_id, *index, network, peer_manager, counter).await
        {
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
        vec![TypeId::of::<PeerManagerWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let requested_messages: RequestedMessages = Default::default();
        node.register_resource(requested_messages);
        let requested_messages = node.resource::<RequestedMessages>();
        let network = node.resource::<NetworkController>();
        let peer_manager = node.resource::<PeerManager>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            let mut counter: usize = 0;
            let mut timeouts = interval(Duration::from_secs(RETRY_INTERVAL_SEC)).fuse();

            loop {
                select! {
                    _ = timeouts.next() => retry_requests(&network, &*requested_messages, &peer_manager, &mut counter).await,
                    entry = receiver.next() => match entry {
                        Some(MessageRequesterWorkerEvent(message_id, index)) => {
                            trace!("Requesting message {}.", message_id);
                            process_request(message_id, index, &network, &peer_manager, &*requested_messages, &mut counter).await
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
