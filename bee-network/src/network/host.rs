// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use crate::{
    alias,
    peer::{
        ban::{AddrBanlist, PeerBanlist},
        store::PeerList,
    },
    service::{
        command::{Command, CommandReceiver},
        event::{InternalEvent, InternalEventSender},
    },
    swarm::{behavior::SwarmBehavior, builder::build_swarm, protocols::gossip::GOSSIP_ORIGIN},
    types::PeerInfo,
};

use libp2p::{
    identity::{self, Keypair},
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm,
};
use log::*;

// TODO: move this to `config` module
// TODO: rename fields: keys, bind, peerlist, ...
pub struct NetworkHostConfig {
    pub local_keys: Keypair,
    pub bind_multiaddr: Multiaddr,
    pub peerlist: PeerList,
    pub banned_addrs: AddrBanlist,
    pub banned_peers: PeerBanlist,
    pub internal_event_sender: InternalEventSender,
    pub internal_command_receiver: CommandReceiver,
}

#[cfg(all(feature = "integrated", not(feature = "standalone")))]
pub mod integrated {
    use super::*;
    use crate::service::service::integrated::NetworkService;

    use bee_runtime::{node::Node, worker::Worker};

    use async_trait::async_trait;

    use std::{any::TypeId, convert::Infallible, sync::atomic::Ordering};

    /// A node worker, that deals with accepting and initiating connections with remote peers.
    /// NOTE: This type is only exported to be used as a worker dependency.
    #[derive(Default)]
    pub struct NetworkHost {}

    #[async_trait]
    impl<N: Node> Worker<N> for NetworkHost {
        type Config = NetworkHostConfig;
        type Error = Infallible;

        fn dependencies() -> &'static [TypeId] {
            vec![TypeId::of::<NetworkService>()].leak()
        }

        async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
            let NetworkHostConfig {
                local_keys,
                bind_multiaddr,
                peerlist,
                banned_addrs,
                banned_peers,
                internal_event_sender,
                mut internal_command_receiver,
            } = config;

            let mut swarm = start_swarm(local_keys.clone(), bind_multiaddr, internal_event_sender.clone()).await;

            node.spawn::<Self, _, _>(|mut shutdown| async move {
                loop {
                    let swarm_next_event = Swarm::next_event(&mut swarm);
                    let recv_command = (&mut internal_command_receiver).recv();

                    tokio::select! {
                        _ = &mut shutdown => break,
                        event = swarm_next_event => {
                            process_swarm_event(event, &internal_event_sender);
                        }
                        command = recv_command => {
                            if let Some(command) = command {
                                process_command(command, &mut swarm, &local_keys, &peerlist, &banned_addrs, &banned_peers).await;
                            }
                        },
                    }
                }

                info!("Network Host stopped.");
            });

            info!("Network Host started.");

            Ok(Self::default())
        }
    }
}

#[cfg(all(feature = "standalone", not(feature = "integrated")))]
pub mod standalone {
    use super::*;
    use crate::service::service::standalone::NetworkService;

    use futures::channel::oneshot;

    pub struct NetworkHost {
        pub shutdown: oneshot::Receiver<()>,
    }

    impl NetworkHost {
        pub fn new(shutdown: oneshot::Receiver<()>) -> Self {
            Self { shutdown }
        }

        pub async fn start(self, config: NetworkHostConfig) {
            let NetworkHost { mut shutdown } = self;
            let NetworkHostConfig {
                local_keys,
                bind_multiaddr,
                peerlist,
                banned_addrs,
                banned_peers,
                internal_event_sender,
                mut internal_command_receiver,
            } = config;

            let mut swarm = start_swarm(local_keys.clone(), bind_multiaddr, internal_event_sender.clone()).await;

            tokio::spawn(async move {
                loop {
                    let swarm_next_event = Swarm::next_event(&mut swarm);
                    let recv_command = (&mut internal_command_receiver).recv();

                    tokio::select! {
                        _ = &mut shutdown => break,
                        event = swarm_next_event => {
                            process_swarm_event(event, &internal_event_sender);
                        }
                        command = recv_command => {
                            if let Some(command) = command {
                                process_command(command, &mut swarm, &local_keys, &peerlist, &banned_addrs, &banned_peers).await;
                            }
                        },
                    }
                }

                info!("Network Host stopped.");
            });

            info!("Network Host started.");
        }
    }
}

async fn start_swarm(
    local_keys: Keypair,
    bind_multiaddr: Multiaddr,
    internal_event_sender: InternalEventSender,
) -> Swarm<SwarmBehavior> {
    let mut swarm = build_swarm(&local_keys, internal_event_sender)
        .await
        .expect("Fatal error: creating transport layer failed.");

    info!("Trying to bind to: {}", bind_multiaddr);

    let _ = Swarm::listen_on(&mut swarm, bind_multiaddr).expect("Fatal error: binding address failed.");

    swarm
}

fn process_swarm_event(event: SwarmEvent<(), impl std::error::Error>, internal_event_sender: &InternalEventSender) {
    match event {
        SwarmEvent::NewListenAddr(address) => {
            // TODO: collect listen address to deny dialing it
            internal_event_sender
                .send(InternalEvent::AddressBound { address })
                .expect("send error");
        }
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            debug!("Negotiating protocol with '{}'.", alias!(peer_id));
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            debug!("Stopped protocol with '{}'.", alias!(peer_id));
        }
        SwarmEvent::ListenerError { error } => {
            error!("Libp2p error: Cause: {}", error);
        }
        SwarmEvent::Dialing(peer_id) => {
            // NB: strange, but this event is not actually fired when dialing. (open issue?)
            debug!("Dialing '{}'.", alias!(peer_id));
        }
        SwarmEvent::IncomingConnection { send_back_addr, .. } => {
            debug!("Being dialed from {}.", send_back_addr);
        }
        _ => {}
    }
}

async fn process_command(
    command: Command,
    swarm: &mut Swarm<SwarmBehavior>,
    local_keys: &Keypair,
    peerlist: &PeerList,
    banned_addrs: &AddrBanlist,
    banned_peers: &PeerBanlist,
) {
    match command {
        Command::DialPeer { peer_id } => {
            if let Err(e) = dial_peer(swarm, local_keys, peer_id, &peerlist, &banned_addrs, &banned_peers).await {
                warn!("Failed to dial peer '...{}'. Cause: {}", alias!(peer_id), e);
            }
        }
        Command::DialAddress { address } => {
            if let Err(e) = dial_addr(swarm, address.clone(), &banned_addrs).await {
                warn!("Failed to dial address '{}'. Cause: {}", address, e);
            }
        }
        _ => {}
    }
}

async fn dial_addr(
    swarm: &mut Swarm<SwarmBehavior>,
    address: Multiaddr,
    banned_addrs: &AddrBanlist,
) -> Result<(), Error> {
    // TODO
    // Prevent dialing own listen addresses.
    // check_if_dialing_own_addr(&address).await?;

    // Prevent dialing banned addresses.
    check_if_banned_addr(&address, &banned_addrs).await?;

    info!("Dialing address {}.", address);
    //
    GOSSIP_ORIGIN.store(true, std::sync::atomic::Ordering::SeqCst);
    Swarm::dial_addr(swarm, address.clone()).map_err(|_| Error::DialingFailed(address))?;

    Ok(())
}

async fn dial_peer(
    swarm: &mut Swarm<SwarmBehavior>,
    local_keys: &identity::Keypair,
    remote_peer_id: PeerId,
    peerlist: &PeerList,
    banned_addrs: &AddrBanlist,
    banned_peers: &PeerBanlist,
) -> Result<(), Error> {
    let local_peer_id = local_keys.public().into_peer_id();

    // Prevent dialing oneself.
    check_if_dialing_self(&local_peer_id, &remote_peer_id).await?;

    // Prevent duplicate connections.
    // NOTE: depending on the ConnectionLimit of the swarm libp2p can catch this as well, but
    // it will be cheaper if we do it here already.
    check_if_duplicate_conn(&remote_peer_id, &peerlist).await?;

    // Prevent dialing banned peers.
    check_if_banned_peer(&remote_peer_id, &banned_peers).await?;

    // Prevent dialing unregistered peers.
    let PeerInfo { address, alias, .. } = check_if_unregistered_or_get_info(&remote_peer_id, &peerlist).await?;

    // TODO
    // Prevent dialing own listen addresses.
    // check_if_dialing_own_addr(&address).await?;

    // Prevent dialing banned addresses.
    check_if_banned_addr(&address, &banned_addrs).await?;

    info!("Dialing peer {}.", alias);

    GOSSIP_ORIGIN.store(true, std::sync::atomic::Ordering::SeqCst);
    Swarm::dial_addr(swarm, address.clone()).map_err(|_| Error::DialingFailed(address))?;

    // // Prevent connecting to dishonest peers or peers we have no up-to-date information about.
    // if received_peer_id != remote_peer_id {
    //     return Err(Error::PeerIdMismatch {
    //         expected: remote_peer_id,
    //         received: received_peer_id,
    //     });
    // }

    Ok(())
}

async fn check_if_dialing_self(local_peer_id: &PeerId, remote_peer_id: &PeerId) -> Result<(), Error> {
    if remote_peer_id.eq(local_peer_id) {
        Err(Error::DialedOwnPeerId(*remote_peer_id))
    } else {
        Ok(())
    }
}

async fn check_if_duplicate_conn(remote_peer_id: &PeerId, peerlist: &PeerList) -> Result<(), Error> {
    if let Ok(true) = peerlist.is(remote_peer_id, |_, state| state.is_connected()).await {
        Err(Error::DuplicateConnection(*remote_peer_id))
    } else {
        Ok(())
    }
}

async fn check_if_banned_peer(remote_peer_id: &PeerId, banned_peers: &PeerBanlist) -> Result<(), Error> {
    if banned_peers.contains(remote_peer_id).await {
        Err(Error::DialedBannedPeer(*remote_peer_id))
    } else {
        Ok(())
    }
}

async fn check_if_unregistered_or_get_info(remote_peer_id: &PeerId, peerlist: &PeerList) -> Result<PeerInfo, Error> {
    peerlist
        .get_info(remote_peer_id)
        .await
        .map_err(|_| Error::DialedUnregisteredPeer(*remote_peer_id))
}

async fn check_if_banned_addr(addr: &Multiaddr, banned_addrs: &AddrBanlist) -> Result<(), Error> {
    if banned_addrs.contains(addr).await {
        Err(Error::DialedBannedAddress(addr.clone()))
    } else {
        Ok(())
    }
}

// TODO: add LISTEN_ADDRESSES
// async fn check_if_dialing_own_addr(addr: &Multiaddr) -> Result<(), Error> {
//     if remote_peer_id.eq(local_peer_id) {
//         Err(super::Error::DialedOwnPeerId(*remote_peer_id))
//     } else {
//         Ok(())
//     }
// }
