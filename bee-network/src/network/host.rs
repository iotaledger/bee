// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use crate::{
    alias,
    peer::list::PeerListWrapper as PeerList,
    service::{
        command::{Command, CommandReceiver},
        event::{InternalEvent, InternalEventSender},
    },
    swarm::{behavior::SwarmBehavior, builder::build_swarm, protocols::gossip::GOSSIP_ORIGIN},
    types::PeerInfo,
};

use libp2p::{identity::Keypair, swarm::SwarmEvent, Multiaddr, PeerId, Swarm};
use log::*;

pub struct NetworkHostConfig {
    pub local_keys: Keypair,
    pub bind_multiaddr: Multiaddr,
    pub internal_event_sender: InternalEventSender,
    pub internal_command_receiver: CommandReceiver,
    pub peerlist: PeerList,
}

pub mod integrated {
    use super::*;
    use crate::service::service::integrated::NetworkService;

    use bee_runtime::{node::Node, worker::Worker};

    use async_trait::async_trait;

    use std::{any::TypeId, convert::Infallible};

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
                internal_event_sender,
                mut internal_command_receiver,
                peerlist,
            } = config;

            let mut swarm = start_swarm(local_keys.clone(), bind_multiaddr, internal_event_sender.clone()).await;

            node.spawn::<Self, _, _>(|mut shutdown| async move {
                loop {
                    let swarm_next_event = Swarm::next_event(&mut swarm);
                    let recv_command = (&mut internal_command_receiver).recv();

                    tokio::select! {
                        _ = &mut shutdown => break,
                        event = swarm_next_event => {
                            process_swarm_event(event, &internal_event_sender, &peerlist).await;
                        }
                        command = recv_command => {
                            if let Some(command) = command {
                                process_command(command, &mut swarm, &peerlist).await;
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

pub mod standalone {
    use super::*;

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
                internal_event_sender,
                mut internal_command_receiver,
                peerlist,
            } = config;

            let mut swarm = start_swarm(local_keys.clone(), bind_multiaddr, internal_event_sender.clone()).await;

            tokio::spawn(async move {
                loop {
                    let swarm_next_event = Swarm::next_event(&mut swarm);
                    let recv_command = (&mut internal_command_receiver).recv();

                    tokio::select! {
                        _ = &mut shutdown => break,
                        event = swarm_next_event => {
                            process_swarm_event(event, &internal_event_sender, &peerlist).await;
                        }
                        command = recv_command => {
                            if let Some(command) = command {
                                process_command(command, &mut swarm, &peerlist).await;
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

    info!("Binding to: {}", bind_multiaddr);

    let _ = Swarm::listen_on(&mut swarm, bind_multiaddr).expect("Fatal error: binding address failed.");

    swarm
}

async fn process_swarm_event(
    event: SwarmEvent<(), impl std::error::Error>,
    internal_event_sender: &InternalEventSender,
    peerlist: &PeerList,
) {
    match event {
        SwarmEvent::NewListenAddr(multiaddr) => {
            debug!("Swarm event: new listen address {}.", multiaddr);

            internal_event_sender
                .send(InternalEvent::AddressBound {
                    address: multiaddr.clone(),
                })
                .expect("send error");

            // INVESTIGATE
            peerlist
                .0
                .write()
                .await
                .insert_local_addr(multiaddr)
                .expect("insert_local_addr");
        }
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            debug!("Swarm event: connection established with {}.", alias!(peer_id));
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            debug!("Swarm event: connection closed with {}.", alias!(peer_id));
        }
        SwarmEvent::ListenerError { error } => {
            error!("Swarm event: listener error {}.", error);
        }
        SwarmEvent::Dialing(peer_id) => {
            // NB: strange, but this event is not actually fired when dialing. (open issue?)
            debug!("Swarm event: dialing {}.", alias!(peer_id));
        }
        SwarmEvent::IncomingConnection { send_back_addr, .. } => {
            debug!("Swarm event: being dialed from {}.", send_back_addr);
        }
        _ => {}
    }
}

async fn process_command(command: Command, swarm: &mut Swarm<SwarmBehavior>, peerlist: &PeerList) {
    match command {
        Command::DialPeer { peer_id } => {
            if let Err(e) = dial_peer(swarm, peer_id, peerlist).await {
                warn!("Failed to dial peer '...{}'. Cause: {}", alias!(peer_id), e);
            }
        }
        Command::DialAddress { address } => {
            if let Err(e) = dial_addr(swarm, address.clone(), peerlist).await {
                warn!("Failed to dial address '{}'. Cause: {}", address, e);
            }
        }
        _ => {}
    }
}

async fn dial_addr(swarm: &mut Swarm<SwarmBehavior>, addr: Multiaddr, peerlist: &PeerList) -> Result<(), Error> {
    if let Err(e) = peerlist.0.read().await.allows_dialing_addr(&addr) {
        warn!("Dialing address {} denied. Cause: {:?}", addr, e);
        return Err(Error::DialingAddressDenied(addr));
    }

    info!("Dialing address: {}.", addr);

    // FIXME
    GOSSIP_ORIGIN.store(true, std::sync::atomic::Ordering::SeqCst);

    Swarm::dial_addr(swarm, addr.clone()).map_err(|_| Error::DialingAddressFailed(addr))?;

    Ok(())
}

async fn dial_peer(swarm: &mut Swarm<SwarmBehavior>, peer_id: PeerId, peerlist: &PeerList) -> Result<(), Error> {
    if let Err(e) = peerlist.0.read().await.allows_dialing_peer(&peer_id) {
        warn!("Dialing peer {} denied. Cause: {:?}", alias!(peer_id), e);
        return Err(Error::DialingPeerDenied(peer_id));
    }

    // Unwrap:
    // We just checked, that the peer is fine to be dialed.
    let PeerInfo { address, alias, .. } = peerlist.0.read().await.info(&peer_id).unwrap();

    info!("Dialing peer: {} ({}).", alias, alias!(peer_id));

    // FIXME
    GOSSIP_ORIGIN.store(true, std::sync::atomic::Ordering::SeqCst);

    Swarm::dial_addr(swarm, address.clone()).map_err(|_| Error::DialingPeerFailed(peer_id))?;

    Ok(())
}
