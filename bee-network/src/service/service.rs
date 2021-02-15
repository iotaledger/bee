// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    commands::{Command, CommandReceiver, CommandSender},
    events::{Event, EventSender, InternalEvent, InternalEventReceiver, InternalEventSender},
};
use crate::{
    alias,
    peer::{self, AddrBanlist, InsertionFailure, PeerBanlist, PeerInfo, PeerList, PeerRelation},
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
pub struct NetworkService {}

pub struct NetworkServiceConfig {
    pub local_keys: identity::Keypair,
    pub peerlist: PeerList,
    pub banned_addrs: AddrBanlist,
    pub banned_peers: PeerBanlist,
    pub event_sender: EventSender,
    pub internal_event_sender: InternalEventSender,
    pub internal_command_sender: CommandSender,
    pub command_receiver: CommandReceiver,
    pub internal_event_receiver: InternalEventReceiver,
}

#[async_trait]
impl<N: Node> Worker<N> for NetworkService {
    type Config = NetworkServiceConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let NetworkServiceConfig {
            local_keys: _,
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
                    &internal_event_sender,
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
    banned_addrlist: &AddrBanlist,
    banned_peerlist: &PeerBanlist,
    event_sender: &EventSender,
    internal_command_sender: &CommandSender,
) -> Result<(), peer::Error> {
    trace!("Received {:?}.", command);

    match command {
        Command::AddPeer {
            peer_id,
            address,
            alias,
            relation,
        } => {
            let alias = alias.unwrap_or_else(|| alias!(peer_id).to_string());

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
                return Err(peer::Error::AddressAlreadyBanned(address));
            }
            if event_sender.send(Event::AddressBanned { address }).is_err() {
                trace!("Failed to send 'AddressBanned' event. (Shutting down?)")
            }
        }
        Command::BanPeer { peer_id } => {
            if !banned_peerlist.insert(peer_id).await {
                return Err(peer::Error::PeerAlreadyBanned(peer_id));
            }

            if event_sender.send(Event::PeerBanned { peer_id }).is_err() {
                trace!("Failed to send 'PeerBanned' event. (Shutting down?)")
            }
        }
        Command::UnbanAddress { address } => {
            if !banned_addrlist.remove(&address).await {
                return Err(peer::Error::AddressAlreadyUnbanned(address));
            }
        }
        Command::UnbanPeer { peer_id } => {
            if !banned_peerlist.remove(&peer_id).await {
                return Err(peer::Error::PeerAlreadyUnbanned(peer_id));
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
    _banned_addrs: &AddrBanlist,
    _banned_peers: &PeerBanlist,
    event_sender: &EventSender,
    _internal_event_sender: &InternalEventSender,
    _internal_command_sender: &CommandSender,
) -> Result<(), peer::Error> {
    trace!("Received {:?}.", internal_event);

    match internal_event {
        InternalEvent::ProtocolEstablished {
            peer_id,
            peer_addr,
            conn_info,
            gossip_in,
            gossip_out,
        } => {
            // In case the peer doesn't exist yet, we create a `PeerInfo` for that peer on-the-fly.
            if !peerlist.contains(&peer_id).await {
                let peer_info = PeerInfo {
                    address: peer_addr,
                    alias: alias!(peer_id).to_string(),
                    relation: PeerRelation::Unknown,
                };

                peerlist
                    .insert(peer_id, peer_info.clone())
                    .await
                    .map_err(|InsertionFailure(_, _, e)| e)?;

                let _ = event_sender.send(Event::PeerAdded {
                    peer_id,
                    info: peer_info,
                });
            }

            debug_assert!(peerlist.contains(&peer_id).await);

            // We can now be sure to always get a `PeerInfo`.
            let peer_info = peerlist
                .get_info(&peer_id)
                .await
                .expect("error getting info although checked");

            peerlist.connect(&peer_id, gossip_out.clone()).await?;

            info!(
                "Established ({}) protocol with '{}'.",
                conn_info.origin, peer_info.alias
            );

            let _ = event_sender.send(Event::PeerConnected {
                peer_id,
                address: peer_info.address,
                gossip_in,
                gossip_out,
            });
        }

        InternalEvent::ProtocolDropped { peer_id } => {
            // NB: Just in case there is any error (PeerMissing, PeerAlreadyDisconnected),
            // then 'disconnect' just becomes a NoOp.

            let _ = peerlist.disconnect(&peer_id).await;

            // NB: In case of an "unknown" peer, we also want it removed from the peerlist. This operation will
            // only fail, if it has been removed already, but will never leave an "unknown" peer in the list.
            let _ = peerlist
                .remove_if(&peer_id, |peer_info, _| peer_info.relation.is_unknown())
                .await;

            let _ = event_sender.send(Event::PeerDisconnected { peer_id });
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
) -> Result<(), peer::Error> {
    let peer_info = PeerInfo {
        address,
        alias,
        relation,
    };

    // If the insert fails for some reason, we get the peer data back, so it can be reused.
    match peerlist.insert(peer_id, peer_info).await {
        Ok(()) => {
            // NB: We could also make `insert` return the just inserted `PeerInfo`, but that would
            // make for an unusual API.

            // We just added it, so 'unwrap'ping is safe!
            let info = peerlist.get_info(&peer_id).await.unwrap();

            let _ = event_sender.send(Event::PeerAdded { peer_id, info });

            Ok(())
        }
        Err(InsertionFailure(peer_id, peer_info, mut e)) => {
            // NB: This fixes an edge case where an in fact known peer connects before being added by the
            // manual peer manager, and hence, as unknown. In such a case we simply update to the correct
            // info (address, alias, relation).

            if matches!(e, peer::Error::PeerAlreadyAdded(_)) && peer_info.relation.is_known() {
                match peerlist.update_info(&peer_id, peer_info.clone()).await {
                    Ok(()) => {
                        let _ = event_sender.send(Event::PeerAdded {
                            peer_id,
                            info: peer_info,
                        });

                        return Ok(());
                    }
                    Err(error) => e = error,
                }
            }

            let _ = event_sender.send(Event::CommandFailed {
                command: Command::AddPeer {
                    peer_id,
                    address: peer_info.address,
                    // NOTE: the returned failed command now has the default alias, if none was specified originally.
                    alias: Some(peer_info.alias),
                    relation: peer_info.relation,
                },
                reason: e.clone(),
            });

            Err(e)
        }
    }
}

async fn remove_peer(peer_id: PeerId, peerlist: &PeerList, event_sender: &EventSender) -> Result<(), peer::Error> {
    disconnect_peer(peer_id, peerlist, event_sender).await?;

    match peerlist.remove(&peer_id).await {
        Ok(_peer_info) => {
            let _ = event_sender.send(Event::PeerRemoved { peer_id });

            Ok(())
        }
        Err(e) => {
            let _ = event_sender.send(Event::CommandFailed {
                command: Command::RemovePeer { peer_id },
                reason: e.clone(),
            });

            Err(e)
        }
    }
}

async fn disconnect_peer(peer_id: PeerId, peerlist: &PeerList, event_sender: &EventSender) -> Result<(), peer::Error> {
    // NB: We sent the `PeerDisconnected` event *before* we sent the shutdown signal to the stream writer task, so
    // it can stop adding messages to the channel before we drop the receiver.

    match peerlist.disconnect(&peer_id).await {
        Ok(gossip_sender) => {
            let _ = event_sender.send(Event::PeerDisconnected { peer_id });

            let shutdown_msg = Vec::with_capacity(0);

            // In very weird situations where both peers disconnect from each other at almost the same time, we
            // might not be able to send to this channel anylonger, so we ignore `SendError`s.
            let _ = gossip_sender.send(shutdown_msg);

            Ok(())
        }
        Err(e) => {
            let _ = event_sender.send(Event::CommandFailed {
                command: Command::DisconnectPeer { peer_id },
                reason: e.clone(),
            });

            Err(e)
        }
    }
}
