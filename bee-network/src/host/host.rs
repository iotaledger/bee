// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::Error;
use crate::{
    alias,
    peer::{AddrBanlist, PeerBanlist, PeerInfo, PeerList},
    service::{HostCommand, HostCommandReceiver, NetworkService, SwarmEventSender},
    swarm,
    swarm::{protocols::gossip::GOSSIP_ORIGIN, SwarmBehavior},
};

use bee_runtime::{node::Node, worker::Worker};

use async_trait::async_trait;
use libp2p::{
    identity::{self, Keypair},
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm,
};
use log::*;

use std::{any::TypeId, convert::Infallible, ops::DerefMut, sync::atomic::Ordering};

pub struct NetworkHostConfig {
    pub local_keys: Keypair,
    pub bind_address: Multiaddr,
    pub peerlist: PeerList,
    pub banned_addrs: AddrBanlist,
    pub banned_peers: PeerBanlist,
    pub swarm_event_sender: SwarmEventSender,
    pub host_command_receiver: HostCommandReceiver,
    pub entry_nodes: Vec<Multiaddr>,
}

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
            bind_address,
            peerlist,
            banned_addrs,
            banned_peers,
            swarm_event_sender,
            mut host_command_receiver,
            entry_nodes,
        } = config;

        let local_keys_clone = local_keys.clone();
        let swarm_event_sender_clone = swarm_event_sender.clone();

        let mut swarm = swarm::build_swarm(&local_keys_clone, swarm_event_sender_clone, entry_nodes)
            .await
            .expect("Fatal error: creating transport layer failed.");

        info!("Binding address(es): {}", bind_address);

        let _ = Swarm::listen_on(&mut swarm, bind_address).expect("Fatal error: address binding failed.");

        {
            let swarm_behavior: &mut crate::swarm::SwarmBehavior = swarm.deref_mut();

            // TEMP
            info!(
                "Kademlia protocol name: {}",
                String::from_utf8_lossy(swarm_behavior.kademlia.protocol_name())
            );

            if let Err(_) = swarm_behavior.kademlia.bootstrap() {
                info!("Running node without DHT entry nodes.");
            }
        }

        node.spawn::<Self, _, _>(|mut shutdown| async move {

            loop {
                let swarm_next_event = Swarm::next_event(&mut swarm);
                // let swarm_next = Swarm::next(&mut swarm);
                let recv_command = (&mut host_command_receiver).recv();

                tokio::select! {
                    // Break on shutdown signal
                    _ = &mut shutdown => break,
                    // Process swarm event
                    event = swarm_next_event => {
                        process_swarm_event(event, &mut swarm, &swarm_event_sender);
                    }
                    // _ = swarm_next => {
                    //     unreachable!();
                    // }
                    // Process command
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

// **Note**: Handles only basic libp2p swarm events mostly relevant for logging. The more specific behavior events
// should be handled in `behavior.rs`.
fn process_swarm_event(
    event: SwarmEvent<(), impl std::error::Error>,
    _swarm: &mut Swarm<SwarmBehavior>,
    _internal_event_sender: &SwarmEventSender,
) {
    match event {
        SwarmEvent::NewListenAddr(_address) => {
            // TODO: collect listen address to deny dialing it
        }
        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
            let address = endpoint.get_remote_address();
            debug!(
                "Underlying transport established with '{}' [{}].",
                alias!(peer_id),
                address,
            );
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            debug!("Closed transport with '{}'.", alias!(peer_id));
        }
        SwarmEvent::ListenerError { error } => {
            error!("Libp2p error: Cause: {}", error);
        }
        SwarmEvent::Dialing(peer_id) => {
            // NB: strange, but this event is not actually fired when dialing. (open issue?)
            debug!("Dialing peer: '{}'.", alias!(peer_id));
        }
        SwarmEvent::IncomingConnection { send_back_addr, .. } => {
            debug!("Being dialed from address {}.", send_back_addr);
        }
        _ => {}
    }
}

async fn process_command(
    command: HostCommand,
    swarm: &mut Swarm<SwarmBehavior>,
    local_keys: &Keypair,
    peerlist: &PeerList,
    banned_addrs: &AddrBanlist,
    banned_peers: &PeerBanlist,
) {
    match command {
        HostCommand::AddPeer { address, peer_id, .. } => {
            println!("Adding address of {} to routing table.", alias!(peer_id));
            {
                let swarm_behavior: &mut crate::swarm::SwarmBehavior = swarm.deref_mut();
                swarm_behavior.kademlia.add_address(&peer_id, address);
            }
        }
        HostCommand::DialPeer { peer_id } => {
            if let Err(e) = dial_peer(swarm, local_keys, peer_id, &peerlist, &banned_addrs, &banned_peers).await {
                warn!("Failed to dial peer '...{}'. Cause: {}", alias!(peer_id), e);
            }
        }
        HostCommand::DialAddress { address } => {
            if let Err(e) = dial_addr(swarm, address.clone(), &banned_addrs).await {
                warn!("Failed to dial address '{}'. Cause: {}", address, e);
            }
        }
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
    GOSSIP_ORIGIN.store(true, Ordering::SeqCst);
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

    GOSSIP_ORIGIN.store(true, Ordering::SeqCst);
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
        Err(super::Error::DialedOwnPeerId(*remote_peer_id))
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
