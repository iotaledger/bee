// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    commands::{Command, CommandReceiver, CommandSender},
    events::{Event, EventSender, InternalEvent, InternalEventReceiver, InternalEventSender},
};
use crate::{
    alias,
    peers::{self, BannedAddrList, BannedPeerList, PeerInfo, PeerList, PeerRelation, PeerState},
    RECONNECT_INTERVAL_SECS,
};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use libp2p::{identity, Multiaddr, PeerId};
use log::*;
use rand::Rng;
use tokio::time::{self, Duration, Instant};
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};

use std::{any::TypeId, convert::Infallible, sync::atomic::Ordering};

/// A node worker, that deals with processing user commands, and publishing events.
/// NOTE: This type is only exported to be used as a worker dependency.
#[derive(Default)]
pub struct Service {}

pub struct ServiceConfig {
    pub local_keys: identity::Keypair,
    pub peerlist: PeerList,
    pub banned_addrs: BannedAddrList,
    pub banned_peers: BannedPeerList,
    pub event_sender: EventSender,
    pub internal_event_sender: InternalEventSender,
    pub internal_command_sender: CommandSender,
    pub command_receiver: CommandReceiver,
    pub internal_event_receiver: InternalEventReceiver,
}

#[async_trait]
impl<N: Node> Worker<N> for Service {
    type Config = ServiceConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let ServiceConfig {
            local_keys,
            peerlist,
            banned_addrs,
            banned_peers,
            event_sender,
            internal_event_sender,
            internal_command_sender,
            command_receiver,
            internal_event_receiver,
        } = config;

        let peerlist_clone = peerlist.clone();
        let banned_addrlist_clone = banned_addrs.clone();
        let banned_peerlist_clone = banned_peers.clone();
        let event_sender_clone = event_sender.clone();
        let internal_command_sender_clone = internal_command_sender.clone();

        // Spawn command handler task
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Command handler running.");

            let mut commands = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(command_receiver));

            while let Some(command) = commands.next().await {
                if let Err(e) = process_command(
                    command,
                    &peerlist_clone,
                    &banned_addrlist_clone,
                    &banned_peerlist_clone,
                    &event_sender_clone,
                    &internal_command_sender_clone,
                )
                .await
                {
                    error!("Error processing command. Cause: {}", e);
                    continue;
                }
            }

            info!("Command handler stopped.");
        });

        let peerlist_clone = peerlist.clone();
        let internal_event_sender_clone = internal_event_sender.clone();
        let internal_command_sender_clone = internal_command_sender.clone();

        // Spawn internal event handler task
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Event handler running.");

            let mut internal_events =
                ShutdownStream::new(shutdown, UnboundedReceiverStream::new(internal_event_receiver));

            while let Some(internal_event) = internal_events.next().await {
                if let Err(e) = process_internal_event(
                    internal_event,
                    &peerlist_clone,
                    &banned_addrs,
                    &banned_peers,
                    &event_sender,
                    &internal_event_sender_clone,
                    &internal_command_sender_clone,
                )
                .await
                {
                    error!("Error processing internal event. Cause: {}", e);
                    continue;
                }
            }

            info!("Event handler stopped.");
        });

        // Spawn reconnecter task
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Reconnecter running.");

            // NOTE: we add a random amount of milliseconds to when the reconnector starts, so that even if 2 nodes
            // go online at the same time the probablilty of them simultaneously dialing each other is reduced
            // significantly.
            // TODO: remove magic number
            let random_delay = rand::thread_rng().gen_range(0u64..5000);
            let start = Instant::now()
                + Duration::from_millis(RECONNECT_INTERVAL_SECS.load(Ordering::Relaxed) * 1000 + random_delay);

            let mut connected_check_timer = ShutdownStream::new(
                shutdown,
                IntervalStream::new(time::interval_at(
                    start,
                    Duration::from_secs(RECONNECT_INTERVAL_SECS.load(Ordering::Relaxed)),
                )),
            );

            while connected_check_timer.next().await.is_some() {
                // Check, if there are any disconnected known peers, and schedule a reconnect attempt for each
                // of those.
                for peer_id in peerlist
                    .iter_if(|info, state| info.relation.is_known() && state.is_disconnected())
                    .await
                {
                    let _ = internal_command_sender.send(Command::DialPeer { peer_id });
                }
            }

            info!("Reconnecter stopped.");
        });

        info!("Network service started.");

        Ok(Self::default())
    }
}

async fn process_command(
    command: Command,
    peerlist: &PeerList,
    banned_addrlist: &BannedAddrList,
    banned_peerlist: &BannedPeerList,
    event_sender: &EventSender,
    internal_command_sender: &CommandSender,
) -> Result<(), peers::Error> {
    trace!("Received {:?}.", command);

    match command {
        Command::AddPeer {
            peer_id,
            address,
            alias,
            relation,
        } => {
            let alias = alias.unwrap_or(alias!(peer_id).to_string());

            // Note: the control flow seems to violate DRY principle, but we only need to clone `id` in one branch.
            if relation.is_known() {
                add_peer(peer_id, address, alias, relation, peerlist, event_sender).await?;

                // We automatically connect to such peers. Since we can connect concurrently, we spawn a task here.
                let _ = internal_command_sender.send(Command::DialPeer { peer_id });
            } else {
                add_peer(peer_id, address, alias, relation, peerlist, event_sender).await?;
            }
        }
        Command::RemovePeer { peer_id } => {
            remove_peer(peer_id, peerlist, event_sender).await?;
        }
        Command::DialPeer { peer_id } => {
            let _ = internal_command_sender.send(Command::DialPeer { peer_id });
        }
        Command::DialAddress { address } => {
            let _ = internal_command_sender.send(Command::DialAddress { address });
        }
        Command::DisconnectPeer { peer_id } => {
            disconnect_peer(peer_id, peerlist, event_sender).await?;
        }
        Command::BanAddress { address } => {
            if !banned_addrlist.insert(address.clone()).await {
                return Err(peers::Error::AddressAlreadyBanned(address));
            } else {
                if event_sender.send(Event::AddressBanned { address }).is_err() {
                    trace!("Failed to send 'AddressBanned' event. (Shutting down?)")
                }
            }
        }
        Command::BanPeer { peer_id } => {
            if !banned_peerlist.insert(peer_id).await {
                return Err(peers::Error::PeerAlreadyBanned(peer_id));
            } else {
                if event_sender.send(Event::PeerBanned { peer_id }).is_err() {
                    trace!("Failed to send 'PeerBanned' event. (Shutting down?)")
                }
            }
        }
        Command::UnbanAddress { address } => {
            if !banned_addrlist.remove(&address).await {
                return Err(peers::Error::AddressAlreadyUnbanned(address));
            }
        }
        Command::UnbanPeer { peer_id } => {
            if !banned_peerlist.remove(&peer_id).await {
                return Err(peers::Error::PeerAlreadyUnbanned(peer_id));
            }
        }
        Command::UpgradeRelation { peer_id } => {
            peerlist.upgrade_relation(&peer_id).await?;
        }
        Command::DowngradeRelation { peer_id } => {
            peerlist.downgrade_relation(&peer_id).await?;
        }
        Command::DiscoverPeers => {
            // TODO: Peer discovery
            // let _ = internal_command_sender.send(Command::DiscoverPeers);
        }
    }

    Ok(())
}

async fn process_internal_event(
    internal_event: InternalEvent,
    peerlist: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    event_sender: &EventSender,
    internal_event_sender: &InternalEventSender,
    internal_command_sender: &CommandSender,
) -> Result<(), peers::Error> {
    trace!("Received {:?}.", internal_event);

    match internal_event {
        InternalEvent::ConnectionEstablished {
            peer_id,
            peer_addr,
            conn_info,
            gossip_in,
            gossip_out,
        } => {
            // Get the PeerInfo, or create one
            let peer_info = if let Ok(peer_info) = peerlist.get_info(&peer_id).await {
                peer_info
            } else {
                PeerInfo {
                    address: peer_addr,
                    alias: alias!(peer_id).to_string(),
                    relation: PeerRelation::Unknown,
                }
            };

            match peer_info.relation {
                PeerRelation::Known => peerlist.update_state(&peer_id, PeerState::Connected).await?,
                PeerRelation::Unknown => {
                    peerlist
                        .insert(peer_id.clone(), peer_info.clone(), PeerState::Connected)
                        .await
                        .map_err(|(_, _, e)| e)?;

                    if event_sender
                        .send(Event::PeerAdded {
                            peer_id,
                            info: peer_info.clone(),
                        })
                        .is_err()
                    {
                        trace!("Error sending event 'PeerAdded'. (Shutting down?)");
                    }
                }
            }

            info!(
                "Established ({}) connection with '{}'.",
                conn_info.origin, peer_info.alias
            );

            if event_sender
                .send(Event::PeerConnected {
                    peer_id,
                    address: peer_info.address,
                    gossip_in,
                    gossip_out,
                })
                .is_err()
            {
                trace!("Error sending event 'PeerConnected'. (Shutting down?)");
            }
        }

        InternalEvent::ConnectionDropped { peer_id } => {
            if let Err(peers::Error::UnregisteredPeer(_)) =
                peerlist.update_state(&peer_id, PeerState::Disconnected).await
            {
                // NOTE: the peer has been removed already
                return Ok(());
            }

            // TODO: maybe allow some fixed timespan for a connection recovery from either end before removing.
            peerlist.remove_if(&peer_id, |info, _| info.relation.is_unknown()).await;

            // NOTE: During shutdown the protocol layer shuts down before the network service, so sending would fail,
            // hence we need to silently ignore send failures until we have a way to query current node state (e.g.
            // NodeState::ShuttingDown)
            if event_sender.send(Event::PeerDisconnected { peer_id }).is_err() {
                trace!("Error sending event 'PeerDisconnected'. (Shutting down?)");
            }
        }
    }

    Ok(())
}

async fn add_peer(
    peer_id: PeerId,
    address: Multiaddr,
    alias: String,
    relation: PeerRelation,
    peerlist: &PeerList,
    event_sender: &EventSender,
) -> Result<(), peers::Error> {
    let info = PeerInfo {
        address,
        alias,
        relation,
    };

    // If the insert fails for some reason, we get the peer info back.
    if let Err((peer_id, info, e)) = peerlist.insert(peer_id, info.clone(), PeerState::Disconnected).await {
        // Inform the user that the command failed.
        let _ = event_sender.send(Event::CommandFailed {
            command: Command::AddPeer {
                peer_id,
                address: info.address,
                // NOTE: the returned failed command now has the default alias, if none was specified originally.
                alias: Some(info.alias),
                relation: info.relation,
            },
            reason: e.clone(),
        });

        return Err(e);
    }

    // Inform the user that the command succeeded.
    if event_sender.send(Event::PeerAdded { peer_id, info }).is_err() {
        warn!("Failed to send 'PeerAdded' event. (Shutting down?)");
    }

    Ok(())
}

async fn remove_peer(peer_id: PeerId, peerlist: &PeerList, event_sender: &EventSender) -> Result<(), peers::Error> {
    match peerlist.remove(&peer_id).await {
        Err(e) => {
            // Inform the user that the command failed.
            let _ = event_sender.send(Event::CommandFailed {
                command: Command::RemovePeer { peer_id },
                reason: e.clone(),
            });

            Err(e)
        }
        Ok(_) => {
            // Inform the user that the command succeeded.
            if event_sender.send(Event::PeerRemoved { peer_id }).is_err() {
                // .map_err(|_| peers::Error::EventSendFailure("PeerRemoved"))?;
            }

            Ok(())
        }
    }
}

async fn disconnect_peer(peer_id: PeerId, peerlist: &PeerList, event_sender: &EventSender) -> Result<(), peers::Error> {
    // NOTE: that's a bit wonky now since we don't own the message sender anymore, but instead pass it to the consumer
    // we propably should intercept
    match peerlist.update_state(&peer_id, PeerState::Disconnected).await {
        Err(e) => {
            // Inform the user that the command failed.
            let _ = event_sender.send(Event::CommandFailed {
                command: Command::DisconnectPeer { peer_id },
                reason: e.clone(),
            });

            Err(e)
        }
        Ok(()) => {
            // Inform the user that the command succeeded.
            let _ = event_sender.send(Event::PeerDisconnected { peer_id });

            Ok(())
        }
    }
}
