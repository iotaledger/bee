// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conns,
    interaction::{
        commands::{Command, CommandReceiver},
        events::{Event, EventSender, InternalEvent, InternalEventReceiver, InternalEventSender},
    },
    peers::{PeerRelation, PeerState},
    ShortId, RECONNECT_INTERVAL_SECS,
};

use super::{errors::Error, BannedAddrList, BannedPeerList, PeerInfo, PeerList};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use libp2p::{identity, Multiaddr, PeerId};
use log::*;
use tokio::time::{self, Duration, Instant};
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};

use std::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
};

pub static NUM_COMMAND_PROCESSING_ERRORS: AtomicUsize = AtomicUsize::new(0);
pub static NUM_EVENT_PROCESSING_ERRORS: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
pub struct PeerManager {}

pub struct PeerManagerConfig {
    local_keys: identity::Keypair,
    event_sender: EventSender,
    internal_event_sender: InternalEventSender,
    peers: PeerList,
    banned_addrs: BannedAddrList,
    banned_peers: BannedPeerList,
    command_receiver: CommandReceiver,
    internal_event_receiver: InternalEventReceiver,
}

impl PeerManagerConfig {
    pub fn new(
        local_keys: identity::Keypair,
        peers: PeerList,
        banned_addrs: BannedAddrList,
        banned_peers: BannedPeerList,
        event_sender: EventSender,
        internal_event_sender: InternalEventSender,
        command_receiver: CommandReceiver,
        internal_event_receiver: InternalEventReceiver,
    ) -> Self {
        Self {
            local_keys,
            peers,
            banned_addrs,
            banned_peers,
            event_sender,
            internal_event_sender,
            command_receiver,
            internal_event_receiver,
        }
    }
}

#[async_trait]
impl<N: Node> Worker<N> for PeerManager {
    type Config = PeerManagerConfig;
    type Error = Infallible;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let PeerManagerConfig {
            local_keys,
            peers,
            banned_addrs,
            banned_peers,
            event_sender,
            internal_event_sender,
            command_receiver,
            internal_event_receiver,
        } = config;

        let local_keys_clone = local_keys.clone();
        let peers_clone = peers.clone();
        let banned_addrs_clone = banned_addrs.clone();
        let banned_peers_clone = banned_peers.clone();
        let event_sender_clone = event_sender.clone();
        let internal_event_sender_clone = internal_event_sender.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Command processor started.");

            let mut commands = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(command_receiver));

            while let Some(command) = commands.next().await {
                if let Err(e) = process_command(
                    command,
                    &local_keys_clone,
                    &peers_clone,
                    &banned_addrs_clone,
                    &banned_peers_clone,
                    &event_sender_clone,
                    &internal_event_sender_clone,
                )
                .await
                {
                    error!("Error processing command. Cause: {}", e);
                    NUM_COMMAND_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            }

            info!("Command processor stopped.");
        });

        let peers_clone = peers.clone();
        let internal_event_sender_clone = internal_event_sender.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Event processor started.");

            let mut internal_events =
                ShutdownStream::new(shutdown, UnboundedReceiverStream::new(internal_event_receiver));

            while let Some(internal_event) = internal_events.next().await {
                if let Err(e) = process_internal_event(
                    internal_event,
                    &local_keys,
                    &peers_clone,
                    &banned_addrs,
                    &banned_peers,
                    &event_sender,
                    &internal_event_sender_clone,
                )
                .await
                {
                    error!("Error processing internal event. Cause: {}", e);
                    NUM_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            }

            info!("Event processor stopped.");
        });

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Reconnector started.");

            let start = Instant::now() + Duration::from_secs(RECONNECT_INTERVAL_SECS.load(Ordering::Relaxed));
            let mut connected_check = ShutdownStream::new(
                shutdown,
                IntervalStream::new(time::interval_at(
                    start,
                    Duration::from_secs(RECONNECT_INTERVAL_SECS.load(Ordering::Relaxed)),
                )),
            );

            while connected_check.next().await.is_some() {
                // Check, if there are any disconnected known peers, and schedule a reconnect attempt for each
                // of those.
                for peer_id in peers
                    .iter_if(|info, state| info.relation.is_known() && state.is_disconnected())
                    .await
                {
                    if let Err(e) = internal_event_sender
                        .send(InternalEvent::ReconnectScheduled { peer_id })
                        .map_err(|_| Error::InternalEventSendFailure("ReconnectScheduled"))
                    {
                        warn!("{:?}", e)
                    }
                }
            }

            info!("Reconnector stopped.");
        });

        trace!("Peer Manager started.");

        Ok(Self::default())
    }
}

async fn process_command(
    command: Command,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    event_sender: &EventSender,
    internal_event_sender: &InternalEventSender,
) -> Result<(), Error> {
    trace!("Received {:?}.", command);

    match command {
        Command::AddPeer {
            id,
            address,
            alias,
            relation,
        } => {
            let alias = alias.unwrap_or(id.short());

            // Note: the control flow seems to violate DRY principle, but we only need to clone `id` in one branch.
            if relation == PeerRelation::Known {
                add_peer(id.clone(), address, alias, relation, peers, event_sender).await?;

                let local_keys = local_keys.clone();
                let peers = peers.clone();
                let banned_addrs = banned_addrs.clone();
                let banned_peers = banned_peers.clone();
                let event_sender = event_sender.clone();
                let internal_event_sender = internal_event_sender.clone();

                // We automatically connect to known peers. Since we can connect concurrently, we spawn a task here.
                tokio::spawn(async move {
                    if let Err(e) = connect_peer(
                        id,
                        local_keys,
                        peers,
                        banned_addrs,
                        banned_peers,
                        internal_event_sender,
                        event_sender,
                    )
                    .await
                    {
                        warn!("Failed to connect to peer. Cause: {:?}", e);
                    }
                });
            } else {
                add_peer(id, address, alias, relation, peers, event_sender).await?;
            }
        }
        Command::RemovePeer { id } => {
            remove_peer(id, peers, event_sender).await?;
        }
        Command::ConnectPeer { id } => {
            let local_keys = local_keys.clone();
            let peers = peers.clone();
            let banned_addrs = banned_addrs.clone();
            let banned_peers = banned_peers.clone();
            let event_sender = event_sender.clone();
            let internal_event_sender = internal_event_sender.clone();

            tokio::spawn(async move {
                if let Err(e) = connect_peer(
                    id,
                    local_keys,
                    peers,
                    banned_addrs,
                    banned_peers,
                    internal_event_sender,
                    event_sender,
                )
                .await
                {
                    warn!("Failed to connect to peer. Cause: {:?}", e);
                }
            });
        }
        Command::DisconnectPeer { id } => {
            disconnect_peer(id, peers, event_sender).await?;
        }
        Command::DialAddress { address } => {
            let local_keys = local_keys.clone();
            let peers = peers.clone();
            let banned_addrs = banned_addrs.clone();
            let banned_peers = banned_peers.clone();
            let event_sender = event_sender.clone();
            let internal_event_sender = internal_event_sender.clone();

            tokio::spawn(async move {
                if let Err(e) = dial_address(
                    address,
                    local_keys,
                    peers,
                    banned_addrs,
                    banned_peers,
                    internal_event_sender,
                    event_sender,
                )
                .await
                {
                    warn!("Failed to connect to peer. Cause: {:?}", e);
                }
            });
        }
        Command::SendMessage { message, to } => {
            send_message(message, &to, peers).await?;
        }
        Command::BanAddress { address } => {
            if !banned_addrs.insert(address.to_string()) {
                return Err(Error::AddressAlreadyBanned(address));
            } else {
                event_sender
                    .send(Event::AddressBanned { address })
                    .map_err(|_| Error::EventSendFailure("AddressBanned"))?;
            }
        }
        Command::BanPeer { id } => {
            if !banned_peers.insert(id.clone()) {
                return Err(Error::PeerAlreadyBanned(id.short()));
            } else {
                event_sender
                    .send(Event::PeerBanned { id })
                    .map_err(|_| Error::EventSendFailure("PeerBanned"))?;
            }
        }
        Command::UnbanAddress { address } => {
            if !banned_addrs.remove(&address.to_string()) {
                return Err(Error::AddressAlreadyUnbanned(address));
            }
        }
        Command::UnbanPeer { id } => {
            if !banned_peers.remove(&id) {
                return Err(Error::PeerAlreadyUnbanned(id.short()));
            }
        }
        Command::UpdateRelation { id, relation } => {
            peers.update_relation(&id, relation).await?;
        }
    }

    Ok(())
}

#[inline]
async fn process_internal_event(
    internal_event: InternalEvent,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    event_sender: &EventSender,
    internal_event_sender: &InternalEventSender,
) -> Result<(), Error> {
    trace!("Received {:?}.", internal_event);

    match internal_event {
        InternalEvent::ConnectionEstablished {
            peer_id,
            peer_info,
            message_sender,
            ..
        } => {
            match peer_info.relation {
                PeerRelation::Known => {
                    peers
                        .update_state(&peer_id, PeerState::Connected(message_sender))
                        .await?
                }
                PeerRelation::Unknown => {
                    peers
                        .insert(peer_id.clone(), peer_info.clone(), PeerState::Connected(message_sender))
                        .await
                        .map_err(|(_, _, e)| e)?;

                    event_sender
                        .send(Event::PeerAdded {
                            id: peer_id.clone(),
                            info: peer_info.clone(),
                        })
                        .map_err(|_| Error::EventSendFailure("PeerAdded"))?;
                }
                // Ignore 'PeerRelation::Discovered' case until autopeering has landed.
                _ => (),
            }

            event_sender
                .send(Event::PeerConnected {
                    id: peer_id,
                    address: peer_info.address,
                })
                .map_err(|_| Error::EventSendFailure("PeerConnected"))?;
        }

        InternalEvent::ConnectionDropped { peer_id } => {
            peers.update_state(&peer_id, PeerState::Disconnected).await?;

            // TODO: maybe allow some fixed timespan for a connection recovery from either end before removing.
            peers.remove_if(&peer_id, |info, _| info.relation.is_unknown()).await;

            event_sender
                .send(Event::PeerDisconnected { id: peer_id })
                .map_err(|_| Error::EventSendFailure("PeerDisconnected"))?;
        }

        InternalEvent::MessageReceived { message, from } => recv_message(message, from, &event_sender).await?,
        InternalEvent::ReconnectScheduled { peer_id } => {
            let local_keys = local_keys.clone();
            let peers = peers.clone();
            let banned_addrs = banned_addrs.clone();
            let banned_peers = banned_peers.clone();
            let event_sender = event_sender.clone();
            let internal_event_sender = internal_event_sender.clone();

            tokio::spawn(async move {
                if let Err(e) = connect_peer(
                    peer_id,
                    local_keys,
                    peers,
                    banned_addrs,
                    banned_peers,
                    internal_event_sender,
                    event_sender,
                )
                .await
                {
                    warn!("Failed to connect to peer. Cause: {:?}", e);
                }
            });
        }
    }

    Ok(())
}

async fn add_peer(
    id: PeerId,
    address: Multiaddr,
    alias: String,
    relation: PeerRelation,
    peers: &PeerList,
    event_sender: &EventSender,
) -> Result<(), Error> {
    let info = PeerInfo {
        address,
        alias,
        relation,
    };

    // If the insert fails for some reason, we get the peer info back.
    if let Err((id, info, e)) = peers.insert(id.clone(), info.clone(), PeerState::Disconnected).await {
        // Inform the user that the command failed.
        event_sender
            .send(Event::CommandFailed {
                command: Command::AddPeer {
                    id,
                    address: info.address,
                    // NOTE: the returned failed command now has the default alias, if none was specified originally.
                    alias: Some(info.alias),
                    relation: info.relation,
                },
            })
            .map_err(|_| Error::EventSendFailure("CommandFailed"))?;

        return Err(e);
    }

    // Inform the user that the command succeeded.
    event_sender
        .send(Event::PeerAdded { id, info })
        .map_err(|_| Error::EventSendFailure("PeerAdded"))?;

    Ok(())
}

async fn remove_peer(id: PeerId, peers: &PeerList, event_sender: &EventSender) -> Result<(), Error> {
    match peers.remove(&id).await {
        Err(e) => {
            // Inform the user that the command failed.
            event_sender
                .send(Event::CommandFailed {
                    command: Command::RemovePeer { id },
                })
                .map_err(|_| Error::EventSendFailure("CommandFailed"))?;

            Err(e)
        }
        Ok(_) => {
            // Inform the user that the command succeeded.
            event_sender
                .send(Event::PeerRemoved { id })
                .map_err(|_| Error::EventSendFailure("PeerRemoved"))?;

            Ok(())
        }
    }
}

async fn connect_peer(
    id: PeerId,
    local_keys: identity::Keypair,
    peers: PeerList,
    banned_addrs: BannedAddrList,
    banned_peers: BannedPeerList,
    internal_event_sender: InternalEventSender,
    event_sender: EventSender,
) -> Result<(), Error> {
    // Try to reach the peer by its known ID.
    if let Err(e) = conns::dial_peer(
        &id,
        &local_keys,
        &internal_event_sender,
        &peers,
        &banned_addrs,
        &banned_peers,
    )
    .await
    .map_err(Error::ConnectFailure)
    {
        // Inform the user that the command failed.
        event_sender
            .send(Event::CommandFailed {
                command: Command::ConnectPeer { id },
            })
            .map_err(|_| Error::EventSendFailure("CommandFailed"))?;

        return Err(e);
    }

    Ok(())
}

async fn disconnect_peer(id: PeerId, peers: &PeerList, event_sender: &EventSender) -> Result<(), Error> {
    match peers.update_state(&id, PeerState::Disconnected).await {
        Err(e) => {
            // Inform the user that the command failed.
            event_sender
                .send(Event::CommandFailed {
                    command: Command::DisconnectPeer { id },
                })
                .map_err(|_| Error::EventSendFailure("CommandFailed"))?;

            Err(e)
        }
        Ok(()) => {
            // Inform the user that the command succeeded.
            event_sender
                .send(Event::PeerDisconnected { id })
                .map_err(|_| Error::EventSendFailure("PeerDisconnected"))?;

            Ok(())
        }
    }
}

async fn dial_address(
    address: Multiaddr,
    local_keys: identity::Keypair,
    peers: PeerList,
    banned_addrs: BannedAddrList,
    banned_peers: BannedPeerList,
    internal_event_sender: InternalEventSender,
    event_sender: EventSender,
) -> Result<(), Error> {
    // Try to reach a peer by its known address.
    if let Err(e) = conns::dial_address(
        &address,
        &local_keys,
        &internal_event_sender,
        &peers,
        &banned_addrs,
        &banned_peers,
    )
    .await
    .map_err(Error::ConnectFailure)
    {
        // Inform the user that the command failed.
        event_sender
            .send(Event::CommandFailed {
                command: Command::DialAddress { address },
            })
            .map_err(|_| Error::EventSendFailure("CommandFailed"))?;

        return Err(e);
    }

    Ok(())
}

#[inline]
async fn send_message(message: Vec<u8>, to: &PeerId, peers: &PeerList) -> Result<(), Error> {
    peers.send_message(message, to).await
}

#[inline]
async fn recv_message(message: Vec<u8>, from: PeerId, event_sender: &EventSender) -> Result<(), Error> {
    event_sender
        .send(Event::MessageReceived { message, from })
        .map_err(|_| Error::EventSendFailure("MessageReceived"))
}
