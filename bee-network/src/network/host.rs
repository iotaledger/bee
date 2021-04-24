// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use crate::{
    alias,
    init::PEER_LIST,
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
                                process_command(command, &mut swarm, &local_keys).await;
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
                internal_event_sender,
                mut internal_command_receiver,
            } = config;

            #[cfg(test)]
            println!("I'm in a test!!");

            let mut swarm = start_swarm(local_keys.clone(), bind_multiaddr, internal_event_sender.clone()).await;

            tokio::spawn(async move {
                loop {
                    let swarm_next_event = Swarm::next_event(&mut swarm);
                    let recv_command = (&mut internal_command_receiver).recv();

                    tokio::select! {
                        _ = &mut shutdown => break,
                        event = swarm_next_event => {
                            process_swarm_event(event, &internal_event_sender).await;
                        }
                        command = recv_command => {
                            if let Some(command) = command {
                                process_command(command, &mut swarm, &local_keys).await;
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
) {
    match event {
        SwarmEvent::NewListenAddr(address) => {
            internal_event_sender
                .send(InternalEvent::AddressBound {
                    address: address.clone(),
                })
                .expect("send error");

            PEER_LIST
                .get()
                .expect("peerlist get")
                .write()
                .await
                .insert_local_addr(address)
                .expect("insert_local_addr");
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

async fn process_command(command: Command, swarm: &mut Swarm<SwarmBehavior>, local_keys: &Keypair) {
    match command {
        Command::DialPeer { peer_id } => {
            if let Err(e) = dial_peer(swarm, local_keys, peer_id).await {
                warn!("Failed to dial peer '...{}'. Cause: {}", alias!(peer_id), e);
            }
        }
        Command::DialAddress { address } => {
            if let Err(e) = dial_addr(swarm, address.clone()).await {
                warn!("Failed to dial address '{}'. Cause: {}", address, e);
            }
        }
        _ => {}
    }
}

async fn dial_addr(swarm: &mut Swarm<SwarmBehavior>, addr: Multiaddr) -> Result<(), Error> {
    if let Err(e) = PEER_LIST.get().unwrap().read().await.allows_dialing_addr(&addr) {
        warn!("Dialing address {} denied. Cause: {:?}", addr, e);
        return Err(Error::DialingAddressDenied(addr));
    }

    info!("Dialing address: {}.", addr);

    // FIXME
    GOSSIP_ORIGIN.store(true, std::sync::atomic::Ordering::SeqCst);

    Swarm::dial_addr(swarm, addr.clone()).map_err(|_| Error::DialingAddressFailed(addr))?;

    Ok(())
}

async fn dial_peer(
    swarm: &mut Swarm<SwarmBehavior>,
    local_keys: &identity::Keypair,
    peer_id: PeerId,
) -> Result<(), Error> {
    if let Err(e) = PEER_LIST.get().unwrap().read().await.allows_dialing_peer(&peer_id) {
        warn!("Dialing peer {} denied. Cause: {:?}", alias!(peer_id), e);
        return Err(Error::DialingPeerDenied(peer_id));
    }

    // `Unwrap`ing is safe, because we just verified that the peer is accepted.
    let PeerInfo { address, alias, .. } = PEER_LIST.get().unwrap().read().await.info(&peer_id).unwrap();

    info!("Dialing peer: {}.", alias);

    // FIXME
    GOSSIP_ORIGIN.store(true, std::sync::atomic::Ordering::SeqCst);

    Swarm::dial_addr(swarm, address.clone()).map_err(|_| Error::DialingPeerFailed(peer_id))?;

    Ok(())
}
