// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    command::{Command, CommandReceiver, CommandSender},
    event::{Event, EventSender, InternalEvent, InternalEventReceiver, InternalEventSender},
};

use crate::{
    alias,
    init::global::reconnect_interval_secs,
    peer::{error::Error as PeerError, list::PeerListWrapper as PeerList},
    types::{PeerInfo, PeerRelation},
};

use bee_runtime::shutdown_stream::ShutdownStream;

use futures::{channel::oneshot, StreamExt};
use libp2p::{identity, Multiaddr, PeerId};
use log::*;
use rand::Rng;
use tokio::time::{self, Duration, Instant};
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};

pub struct NetworkServiceConfig {
    pub local_keys: identity::Keypair,
    pub senders: Senders,
    pub receivers: Receivers,
    pub peerlist: PeerList,
}

#[derive(Clone)]
pub struct Senders {
    pub events: EventSender,
    pub internal_events: InternalEventSender,
    pub internal_commands: CommandSender,
}

pub struct Receivers {
    pub commands: CommandReceiver,
    pub internal_events: InternalEventReceiver,
}

type Shutdown = oneshot::Receiver<()>;

pub mod integrated {
    use super::*;

    use bee_runtime::{node::Node, worker::Worker};

    use async_trait::async_trait;

    use std::{any::TypeId, convert::Infallible};

    /// A node worker, that deals with processing user commands, and publishing events.
    ///
    /// NOTE: This type is only exported to be used as a worker dependency.
    #[derive(Default)]
    pub struct NetworkService {}

    #[async_trait]
    impl<N: Node> Worker<N> for NetworkService {
        type Config = NetworkServiceConfig;
        type Error = Infallible;

        fn dependencies() -> &'static [TypeId] {
            &[]
        }

        async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
            let NetworkServiceConfig {
                local_keys: _,
                senders,
                receivers,
                peerlist,
            } = config;

            let Receivers {
                commands,
                internal_events,
            } = receivers;

            node.spawn::<Self, _, _>(|shutdown| {
                command_processor(shutdown, commands, senders.clone(), peerlist.clone())
            });
            node.spawn::<Self, _, _>(|shutdown| {
                event_processor(shutdown, internal_events, senders.clone(), peerlist.clone())
            });
            node.spawn::<Self, _, _>(|shutdown| peer_checker(shutdown, senders, peerlist));

            info!("Network service started.");

            Ok(Self::default())
        }
    }
}

pub mod standalone {
    use super::*;

    pub struct NetworkService {
        pub shutdown: oneshot::Receiver<()>,
    }

    impl NetworkService {
        pub fn new(shutdown: oneshot::Receiver<()>) -> Self {
            Self { shutdown }
        }

        pub async fn start(self, config: NetworkServiceConfig) {
            let NetworkService { shutdown } = self;
            let NetworkServiceConfig {
                local_keys: _,
                senders,
                receivers,
                peerlist,
            } = config;

            let Receivers {
                commands,
                internal_events,
            } = receivers;

            let (shutdown_tx1, shutdown_rx1) = oneshot::channel::<()>();
            let (shutdown_tx2, shutdown_rx2) = oneshot::channel::<()>();
            let (shutdown_tx3, shutdown_rx3) = oneshot::channel::<()>();

            tokio::spawn(async move {
                shutdown.await.expect("receiving shutdown signal");

                shutdown_tx1.send(()).expect("receiving shutdown signal");
                shutdown_tx2.send(()).expect("receiving shutdown signal");
                shutdown_tx3.send(()).expect("receiving shutdown signal");
            });
            tokio::spawn(command_processor(
                shutdown_rx1,
                commands,
                senders.clone(),
                peerlist.clone(),
            ));
            tokio::spawn(event_processor(
                shutdown_rx2,
                internal_events,
                senders.clone(),
                peerlist.clone(),
            ));
            tokio::spawn(peer_checker(shutdown_rx3, senders, peerlist));

            info!("Network service started.");
        }
    }
}

async fn command_processor(shutdown: Shutdown, commands: CommandReceiver, senders: Senders, peerlist: PeerList) {
    debug!("Command processor running.");

    let mut commands = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(commands));

    while let Some(command) = commands.next().await {
        if let Err(e) = process_command(command, &senders, &peerlist).await {
            error!("Error processing command. Cause: {}", e);
            continue;
        }
    }

    debug!("Command processor stopped.");
}

async fn event_processor(shutdown: Shutdown, events: InternalEventReceiver, senders: Senders, peerlist: PeerList) {
    debug!("Event processor running.");

    let mut int_events = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(events));

    while let Some(int_event) = int_events.next().await {
        if let Err(e) = process_internal_event(int_event, &senders, &peerlist).await {
            error!("Error processing internal event. Cause: {}", e);
            continue;
        }
    }

    debug!("Event processor stopped.");
}

// TODO: implement exponential back-off to not spam the peer with reconnect attempts.
async fn peer_checker(shutdown: Shutdown, senders: Senders, peerlist: PeerList) {
    debug!("Peer checker running.");

    let Senders { internal_commands, .. } = senders;

    // NOTE:
    // We add a random amount of milliseconds to when the reconnector starts, so that even if 2 nodes
    // go online at the same time the probablilty of them simultaneously dialing each other is reduced
    // significantly.

    let delay = Duration::from_millis(rand::thread_rng().gen_range(0u64..1000));
    let start = Instant::now() + delay;
    let period = Duration::from_secs(reconnect_interval_secs()); // `unwrap` is safe!

    let mut interval = ShutdownStream::new(shutdown, IntervalStream::new(time::interval_at(start, period)));

    // Check, if there are any disconnected known peers, and schedule a reconnect attempt for each
    // of those.
    while interval.next().await.is_some() {
        let peerlist = peerlist.0.read().await;

        for (peer_id, alias) in peerlist.filter(|info, state| info.relation.is_known() && state.is_disconnected()) {
            info!("Trying to reconnect to: {} ({}).", alias, alias!(peer_id));

            // Ignore if the command fails. We can always try another time.
            let _ = internal_commands.send(Command::DialPeer { peer_id });
        }
    }

    debug!("Peer checker stopped.");
}

async fn process_command(command: Command, senders: &Senders, peerlist: &PeerList) -> Result<(), PeerError> {
    trace!("Received {:?}.", command);

    match command {
        Command::AddPeer {
            peer_id,
            multiaddr,
            alias,
            relation,
        } => {
            let alias = alias.unwrap_or_else(|| alias!(peer_id).to_string());

            add_peer(peer_id, multiaddr, alias, relation, senders, peerlist).await?;

            if relation.is_known() {
                // We automatically connect to such peers.
                let _ = senders.internal_commands.send(Command::DialPeer { peer_id });
            }
        }

        Command::BanAddress { address } => {
            let mut peerlist = peerlist.0.write().await;
            peerlist.ban_address(address.clone())?;

            let _ = senders.events.send(Event::AddressBanned { address });
        }

        Command::BanPeer { peer_id } => {
            let mut peerlist = peerlist.0.write().await;
            peerlist.ban_peer(peer_id)?;

            let _ = senders.events.send(Event::PeerBanned { peer_id });
        }

        Command::ChangeRelation { peer_id, to } => {
            let mut peerlist = peerlist.0.write().await;
            peerlist.update_info(&peer_id, |info| info.relation = to)?;
        }

        Command::DialAddress { address } => {
            let _ = senders.internal_commands.send(Command::DialAddress { address });
        }

        Command::DialPeer { peer_id } => {
            let _ = senders.internal_commands.send(Command::DialPeer { peer_id });
        }

        Command::DisconnectPeer { peer_id } => {
            disconnect_peer(peer_id, senders, peerlist).await?;
        }

        Command::RemovePeer { peer_id } => {
            remove_peer(peer_id, senders, peerlist).await?;
        }

        Command::UnbanAddress { address } => {
            let mut peerlist = peerlist.0.write().await;
            peerlist.unban_address(&address)?;

            let _ = senders.events.send(Event::AddressUnbanned { address });
        }

        Command::UnbanPeer { peer_id } => {
            let mut peerlist = peerlist.0.write().await;
            peerlist.unban_peer(&peer_id)?;

            let _ = senders.events.send(Event::PeerUnbanned { peer_id });
        }
    }

    Ok(())
}

async fn process_internal_event(
    int_event: InternalEvent,
    senders: &Senders,
    peerlist: &PeerList,
) -> Result<(), PeerError> {
    match int_event {
        InternalEvent::AddressBound { address } => {
            let _ = senders.events.send(Event::AddressBound { address });
        }

        InternalEvent::ProtocolDropped { peer_id } => {
            let mut peerlist = peerlist.0.write().await;

            // Try to disconnect, but ignore errors in-case the peer was disconnected already.
            let _ = peerlist.update_state(&peer_id, |state| state.to_disconnected());

            // Try to remove unknown peers.
            let _ = peerlist.filter_remove(&peer_id, |peer_info, _| peer_info.relation.is_unknown());

            let _ = senders.events.send(Event::PeerDisconnected { peer_id });
        }

        InternalEvent::ProtocolEstablished {
            peer_id,
            peer_addr,
            conn_info,
            gossip_in,
            gossip_out,
        } => {
            let mut peerlist = peerlist.0.write().await;

            // In case the peer doesn't exist yet, we create a `PeerInfo` for that peer on-the-fly.
            if !peerlist.contains(&peer_id) {
                let peer_info = PeerInfo {
                    address: peer_addr,
                    alias: alias!(peer_id).to_string(),
                    relation: PeerRelation::Unknown,
                };

                peerlist
                    .insert_peer(peer_id, peer_info.clone())
                    .map_err(|(_, _, e)| e)?;

                let _ = senders.events.send(Event::PeerAdded {
                    peer_id,
                    info: peer_info,
                });
            }

            // Panic:
            // We made sure, that the peer id exists in the above if-branch, hence, unwrapping is fine.
            let peer_info = peerlist.info(&peer_id).unwrap();

            // We store a clone of the gossip send channel in order to send a shutdown signal.
            let _ = peerlist.update_state(&peer_id, |state| state.to_connected(gossip_out.clone()));

            info!(
                "Established ({}) protocol with {} ({}).",
                conn_info.origin,
                peer_info.alias,
                alias!(peer_id)
            );

            let _ = senders.events.send(Event::PeerConnected {
                peer_id,
                info: peer_info,
                gossip_in,
                gossip_out,
            });
        }
    }

    Ok(())
}

async fn add_peer(
    peer_id: PeerId,
    address: Multiaddr,
    alias: String,
    relation: PeerRelation,
    senders: &Senders,
    peerlist: &PeerList,
) -> Result<(), PeerError> {
    let peer_info = PeerInfo {
        address,
        alias,
        relation,
    };

    let mut peerlist = peerlist.0.write().await;

    // If the insert fails for some reason, we get the peer data back, so it can be reused.
    match peerlist.insert_peer(peer_id, peer_info) {
        Ok(()) => {
            // NB: We could also make `insert` return the just inserted `PeerInfo`, but that would
            // make for an unusual API.

            // We just added it, so 'unwrap'ping is safe!
            let info = peerlist.info(&peer_id).unwrap();

            let _ = senders.events.send(Event::PeerAdded { peer_id, info });

            Ok(())
        }
        Err((peer_id, peer_info, mut e)) => {
            // NB: This fixes an edge case where an in fact known peer connects before being added by the
            // manual peer manager, and hence, as unknown. In such a case we simply update to the correct
            // info (address, alias, relation).

            // TODO: Since we nowadays add static peers during initialization (`init`), the above mentioned edge case is
            // impossible to happen, and hence this match case can probably be removed. But this needs to be tested
            // thoroughly in a live setup to really be sure.

            if matches!(e, PeerError::PeerIsDuplicate(_)) {
                match peerlist.update_info(&peer_id, |info| *info = peer_info.clone()) {
                    Ok(()) => {
                        let _ = senders.events.send(Event::PeerAdded {
                            peer_id,
                            info: peer_info,
                        });

                        return Ok(());
                    }
                    Err(error) => e = error,
                }
            }

            let _ = senders.events.send(Event::CommandFailed {
                command: Command::AddPeer {
                    peer_id,
                    multiaddr: peer_info.address,
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

async fn remove_peer(peer_id: PeerId, senders: &Senders, peerlist: &PeerList) -> Result<(), PeerError> {
    disconnect_peer(peer_id, senders, peerlist).await?;

    let mut peerlist = peerlist.0.write().await;

    match peerlist.remove(&peer_id) {
        Ok(_peer_info) => {
            let _ = senders.events.send(Event::PeerRemoved { peer_id });

            Ok(())
        }
        Err(e) => {
            let _ = senders.events.send(Event::CommandFailed {
                command: Command::RemovePeer { peer_id },
                reason: e.clone(),
            });

            Err(e)
        }
    }
}

async fn disconnect_peer(peer_id: PeerId, senders: &Senders, peerlist: &PeerList) -> Result<(), PeerError> {
    let mut peerlist = peerlist.0.write().await;

    // NB: We sent the `PeerDisconnected` event *before* we sent the shutdown signal to the stream writer task, so
    // it can stop adding messages to the channel before we drop the receiver.

    match peerlist.update_state(&peer_id, |state| state.to_disconnected()) {
        Ok(Some(gossip_sender)) => {
            let _ = senders.events.send(Event::PeerDisconnected { peer_id });

            // Try to send the shutdown signal. It has to be a Vec<u8>, but it doesn't have to allocate.
            let _ = gossip_sender.send(Vec::new());

            Ok(())
        }
        Ok(None) => {
            // already disconnected
            Ok(())
        }
        Err(e) => {
            let _ = senders.events.send(Event::CommandFailed {
                command: Command::DisconnectPeer { peer_id },
                reason: e.clone(),
            });

            Err(e)
        }
    }
}
