// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    commands::{Command, CommandReceiver, HostCommand, HostCommandSender},
    events::{Event, EventSender, SwarmEvent, SwarmEventReceiver, SwarmEventSender},
};
use crate::{
    alias,
    peer::{self, InsertionFailure, PeerInfo, PeerList, PeerRelation},
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
    pub event_sender: EventSender,
    pub command_receiver: CommandReceiver,
    pub swarm_event_sender: SwarmEventSender,
    pub swarm_event_receiver: SwarmEventReceiver,
    pub host_command_sender: HostCommandSender,
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
            event_sender,
            command_receiver,
            swarm_event_sender,
            swarm_event_receiver,
            host_command_sender,
        } = config;

        let peerlist_clone = peerlist.clone();
        let event_sender_clone = event_sender.clone();
        let host_command_sender_clone = host_command_sender.clone();

        // Spawn command handler task
        node.spawn::<Self, _, _>(|shutdown| async move {
            debug!("Command handler running.");

            let mut commands = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(command_receiver));

            while let Some(command) = commands.next().await {
                if let Err(e) = process_command(
                    command,
                    &peerlist_clone,
                    &event_sender_clone,
                    &host_command_sender_clone,
                )
                .await
                {
                    error!("Error processing command. Cause: {}", e);
                    continue;
                }
            }

            debug!("Command handler stopped.");
        });

        let peerlist_clone = peerlist.clone();
        let host_command_sender_clone = host_command_sender.clone();

        // Spawn internal event handler task
        node.spawn::<Self, _, _>(|shutdown| async move {
            debug!("Event handler running.");

            let mut swarm_events = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(swarm_event_receiver));

            while let Some(swarm_event) = swarm_events.next().await {
                if let Err(e) = process_swarm_event(
                    swarm_event,
                    &peerlist_clone,
                    &event_sender,
                    &swarm_event_sender,
                    &host_command_sender_clone,
                )
                .await
                {
                    error!("Error processing internal event. Cause: {}", e);
                    continue;
                }
            }

            debug!("Event handler stopped.");
        });

        // Spawn reconnecter task
        node.spawn::<Self, _, _>(|shutdown| async move {
            // NOTE: we add a random amount of milliseconds to when the reconnector starts, so that even if 2 nodes
            // go online at the same time the probablilty of them simultaneously dialing each other is reduced
            // significantly.
            // TODO: remove magic number
            let randomized_delay = Duration::from_millis(
                RECONNECT_INTERVAL_SECS.load(Ordering::Relaxed) * 1000 + rand::thread_rng().gen_range(0u64..1000),
            );
            let start = Instant::now() + randomized_delay;

            let mut connected_check_timer = ShutdownStream::new(
                shutdown,
                IntervalStream::new(time::interval_at(
                    start,
                    Duration::from_secs(RECONNECT_INTERVAL_SECS.load(Ordering::Relaxed)),
                )),
            );

            debug!("Reconnecter starting in {:?}.", randomized_delay);

            while connected_check_timer.next().await.is_some() {
                // Check, if there are any disconnected known peers, and schedule a reconnect attempt for each
                // of those.
                for (peer_id, alias) in peerlist
                    .iter_if(|info, state| info.relation.is_known() && state.is_disconnected())
                    .await
                {
                    info!("Reconnecting to {}.", alias);

                    // Not being able to send something over this channel must be considered a bug.
                    host_command_sender
                        .send(HostCommand::DialPeer { peer_id })
                        .expect("Reconnector failed to send 'DialPeer' command.")
                }
            }

            debug!("Reconnecter stopped.");
        });

        info!("Network service started.");

        Ok(Self::default())
    }
}

async fn process_command(
    command: Command,
    peerlist: &PeerList,
    event_sender: &EventSender,
    host_command_sender: &HostCommandSender,
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

            // Add this peer to the peer list.
            add_peer(peer_id, address.clone(), alias, relation, peerlist, event_sender).await?;

            // Add address of this peer to the routing table.
            // TODO: make this optional
            let _ = host_command_sender.send(HostCommand::AddPeerAddrToRoutingTable { peer_id, address });

            // We try to connect to known peers immediatedly.
            if relation.is_known() {
                let _ = host_command_sender.send(HostCommand::DialPeer { peer_id });
            }
        }
        Command::RemovePeer { peer_id } => {
            remove_peer(peer_id, peerlist, event_sender).await?;
        }
        Command::DialPeer { peer_id } => {
            let _ = host_command_sender.send(HostCommand::DialPeer { peer_id });
        }
        Command::DialAddress { address } => {
            let _ = host_command_sender.send(HostCommand::DialAddress { address });
        }
        Command::DisconnectPeer { peer_id } => {
            disconnect_peer(peer_id, peerlist, event_sender).await?;
        }
        Command::BanAddress { address } => {
            peerlist.ban_address(address.clone()).await?;

            if event_sender.send(Event::AddressBanned { address }).is_err() {
                trace!("Failed to send 'AddressBanned' event. (Shutting down?)")
            }
        }
        Command::BanPeer { peer_id } => {
            peerlist.ban_peer(peer_id).await?;

            if event_sender.send(Event::PeerBanned { peer_id }).is_err() {
                trace!("Failed to send 'PeerBanned' event. (Shutting down?)")
            }
        }
        Command::UnbanAddress { address } => {
            peerlist.unban_address(&address).await?;
        }
        Command::UnbanPeer { peer_id } => {
            peerlist.unban_peer(&peer_id).await?;
        }
        Command::UpgradeRelation { peer_id } => {
            peerlist
                .update_info(&peer_id, |info| {
                    info.relation.upgrade();
                })
                .await?;
        }
        Command::DowngradeRelation { peer_id } => {
            peerlist
                .update_info(&peer_id, |info| {
                    info.relation.downgrade();
                })
                .await?;
        }
    }

    Ok(())
}

async fn process_swarm_event(
    swarm_event: SwarmEvent,
    peerlist: &PeerList,
    event_sender: &EventSender,
    _swarm_event_sender: &SwarmEventSender,
    host_command_sender: &HostCommandSender,
) -> Result<(), peer::Error> {
    match swarm_event {
        SwarmEvent::ProtocolEstablished {
            peer_id,
            address,
            conn_info,
            gossip_in,
            gossip_out,
        } => {
            // In case the peer doesn't exist yet, we create a `PeerInfo` for that peer on-the-fly.
            if !peerlist.contains(&peer_id).await {
                let peer_info = PeerInfo {
                    address,
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
                .info(&peer_id)
                .await
                .expect("error getting info although checked");

            let _ = peerlist
                .update_state(&peer_id, |state| state.set_connected(gossip_out.clone()))
                .await?;

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

        SwarmEvent::ProtocolDropped { peer_id } => {
            // NB: Just in case there is any error (PeerMissing, PeerAlreadyDisconnected),
            // then 'disconnect' just becomes a NoOp.

            let _ = peerlist
                .update_state(&peer_id, |state| {
                    // ignore the returned gossip sender
                    state.set_disconnected()
                })
                .await?;

            // Only in case of an *known* peer, we want to keep it in the peerlist. This operation will only fail, if it
            // has been removed already, but will never leave an "unknown" peer in the list.
            let _ = peerlist
                .remove_if(&peer_id, |peer_info, _| !peer_info.relation.is_known())
                .await;

            let _ = event_sender.send(Event::PeerDisconnected { peer_id });
        }

        SwarmEvent::RoutingTableUpdated { peer_id, addresses } => {
            println!("Routing table updated for {} ({} addresses)", peer_id, addresses.len());

            // TODO: also consider other addresses.
            let address = addresses
                .iter()
                .nth(0)
                .expect("peer didn't provide any addresses")
                .clone();

            let peer_info = PeerInfo {
                address,
                alias: alias!(peer_id).to_string(),
                relation: PeerRelation::Discovered,
            };

            peerlist
                .insert(peer_id, peer_info)
                .await
                .map_err(|InsertionFailure(_, _, e)| e)?;

            // // Connect to the discovered peer, if all checks pass
            // if peerlist.accepts(&peer_id, &peer_info).await.is_ok() {
            //     println!("New dialable peer: {}, {:?}", peer_id, peer_info);

            //     host_command_sender
            //         .send(HostCommand::DialAddress {
            //             address: peer_info.address,
            //         })
            //         .expect("command channel receiver dropped");
            // }
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
            let info = peerlist.info(&peer_id).await.unwrap();

            let _ = event_sender.send(Event::PeerAdded { peer_id, info });

            Ok(())
        }
        Err(InsertionFailure(peer_id, peer_info, e)) => {
            // **NB**: This fixes an edge case where an in fact known peer connects before being added by the
            // manual peer manager, and hence, as unknown. In such a case we simply update to the correct
            // info (address, alias, relation).
            // **UPDATE**: This edge case should not happen anymore since `bee-peering` became part of `bee-network`.

            if matches!(e, peer::Error::PeerAlreadyAdded(_)) {
                unreachable!("peer already added edge case");
                // match peerlist
                //     .update(&peer_id, |info, _| {
                //         *info = peer_info.clone();
                //     })
                //     .await
                // {
                //     Ok(()) => {
                //         let _ = event_sender.send(Event::PeerAdded {
                //             peer_id,
                //             info: peer_info,
                //         });

                //         return Ok(());
                //     }
                //     Err(error) => e = error,
                // }
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
    // **NB**: We sent the `PeerDisconnected` event *before* we sent the shutdown signal to the stream writer task, so
    // it can stop adding messages to the channel before we drop the receiver.

    match peerlist.update_state(&peer_id, |state| state.set_disconnected()).await {
        // match peerlist.disconnect(&peer_id).await {
        Ok(Some(gossip_sender)) => {
            let _ = event_sender.send(Event::PeerDisconnected { peer_id });

            let shutdown_msg = Vec::with_capacity(0);

            // In very weird situations where both peers disconnect from each other at almost the same time, we
            // might not be able to send to this channel anylonger, so we ignore `SendError`s.
            let _ = gossip_sender.send(shutdown_msg);

            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => {
            let _ = event_sender.send(Event::CommandFailed {
                command: Command::DisconnectPeer { peer_id },
                reason: e.clone(),
            });

            Err(e)
        }
    }
}
