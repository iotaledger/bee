// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use futures::{channel::oneshot, StreamExt};
use libp2p::{swarm::SwarmEvent, Multiaddr, PeerId, Swarm};
use log::*;

use super::error::Error;
use crate::{
    alias,
    peer::{info::PeerInfo, list::PeerListWrapper as PeerList},
    service::{
        command::{Command, CommandReceiver},
        event::{InternalEvent, InternalEventSender},
    },
    swarm::behaviour::SwarmBehaviour,
};

pub struct NetworkHostConfig {
    pub internal_event_sender: InternalEventSender,
    pub internal_command_receiver: CommandReceiver,
    pub peerlist: PeerList,
    pub swarm: Swarm<SwarmBehaviour>,
    pub bind_multiaddr: Multiaddr,
}

pub mod integrated {
    use std::{any::TypeId, convert::Infallible};

    use async_trait::async_trait;
    use bee_runtime::{node::Node, worker::Worker};

    use super::*;
    use crate::service::host::integrated::ServiceHost;

    /// A node worker, that deals with accepting and initiating connections with remote peers.
    ///
    /// NOTE: This type is only exported to be used as a worker dependency.
    #[derive(Default)]
    pub struct NetworkHost {}

    #[async_trait]
    impl<N: Node> Worker<N> for NetworkHost {
        type Config = NetworkHostConfig;
        type Error = Infallible;

        fn dependencies() -> &'static [TypeId] {
            vec![TypeId::of::<ServiceHost>()].leak()
        }

        #[cfg_attr(feature = "trace", trace_tools::observe)]
        async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
            node.spawn::<Self, _, _>(|shutdown| async move {
                network_host_processor(config, shutdown)
                    .await
                    .expect("network host processor");

                info!("Network Host stopped.");
            });

            info!("Network Host started.");

            Ok(Self::default())
        }
    }
}

pub mod standalone {
    use super::*;

    pub struct NetworkHost {
        pub shutdown: oneshot::Receiver<()>,
    }

    impl NetworkHost {
        pub fn new(shutdown: oneshot::Receiver<()>) -> Self {
            Self { shutdown }
        }

        pub async fn start(self, config: NetworkHostConfig) {
            let NetworkHost { shutdown } = self;

            tokio::spawn(async move {
                network_host_processor(config, shutdown)
                    .await
                    .expect("network host processor");

                info!("Network Host stopped.");
            });

            info!("Network Host started.");
        }
    }
}

#[cfg_attr(feature = "trace", trace_tools::observe)]
async fn network_host_processor(
    config: NetworkHostConfig,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<(), crate::Error> {
    let NetworkHostConfig {
        internal_event_sender,
        mut internal_command_receiver,
        peerlist,
        mut swarm,
        bind_multiaddr,
    } = config;

    // Try binding to the configured bind address.
    info!("Binding to: {}", bind_multiaddr);
    let _listener_id = Swarm::listen_on(&mut swarm, bind_multiaddr).map_err(|_| crate::Error::BindingAddressFailed)?;

    // Enter command/event loop.
    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            event = (&mut swarm).next() => {
                let event = event.ok_or(crate::Error::HostEventLoopError)?;
                process_swarm_event(event, &internal_event_sender, &peerlist).await;
            }
            command = (&mut internal_command_receiver).recv() => {
                let command = command.ok_or(crate::Error::HostEventLoopError)?;
                process_internal_command(command, &mut swarm, &peerlist).await;
            },
        }
    }

    Ok(())
}

#[cfg_attr(feature = "trace", trace_tools::observe)]
async fn process_swarm_event(
    event: SwarmEvent<(), impl std::error::Error>,
    internal_event_sender: &InternalEventSender,
    peerlist: &PeerList,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            debug!("Swarm event: new listen address {}.", address);

            internal_event_sender
                .send(InternalEvent::AddressBound {
                    address: address.clone(),
                })
                .expect("send error");

            // Note: We don't care if the inserted address is a duplicate.
            let _ = peerlist.0.write().await.add_local_addr(address);
        }
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            debug!("Swarm event: connection established with {}.", alias!(peer_id));
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            debug!("Swarm event: connection closed with {}.", alias!(peer_id));
        }
        SwarmEvent::ListenerError { error, .. } => {
            error!("Swarm event: listener error {}.", error);
        }
        SwarmEvent::Dialing(peer_id) => {
            // TODO: strange, but this event is not actually fired when dialing. (open issue?)
            debug!("Swarm event: dialing {}.", alias!(peer_id));
        }
        SwarmEvent::IncomingConnection { send_back_addr, .. } => {
            debug!("Swarm event: being dialed from {}.", send_back_addr);
        }
        _ => {}
    }
}

#[cfg_attr(feature = "trace", trace_tools::observe)]
async fn process_internal_command(internal_command: Command, swarm: &mut Swarm<SwarmBehaviour>, peerlist: &PeerList) {
    match internal_command {
        Command::DialAddress { address } => {
            if let Err(e) = dial_addr(swarm, address.clone(), peerlist).await {
                warn!("Dialing address {} failed. Cause: {}", address, e);
            }
        }
        Command::DialPeer { peer_id } => {
            if let Err(e) = dial_peer(swarm, peer_id, peerlist).await {
                warn!("Dialing peer {} failed. Cause: {}", alias!(peer_id), e);
            }
        }
        Command::DisconnectPeer { peer_id } => hang_up(swarm, peer_id),
        _ => {}
    }
}

#[cfg_attr(feature = "trace", trace_tools::observe)]
async fn dial_addr(swarm: &mut Swarm<SwarmBehaviour>, addr: Multiaddr, peerlist: &PeerList) -> Result<(), Error> {
    if let Err(e) = peerlist.0.read().await.allows_dialing_addr(&addr) {
        warn!("Dialing address {} denied. Cause: {:?}", addr, e);
        return Err(Error::DialingAddressDenied(addr));
    }

    info!("Dialing address: {}.", addr);

    Swarm::dial(swarm, addr.clone()).map_err(|e| Error::DialingAddressFailed(addr, e))?;

    Ok(())
}

#[cfg_attr(feature = "trace", trace_tools::observe)]
async fn dial_peer(swarm: &mut Swarm<SwarmBehaviour>, peer_id: PeerId, peerlist: &PeerList) -> Result<(), Error> {
    if let Err(e) = peerlist.0.read().await.allows_dialing_peer(&peer_id) {
        warn!("Dialing peer {} denied. Cause: {:?}", alias!(peer_id), e);
        return Err(Error::DialingPeerDenied(peer_id));
    }

    // Panic:
    // We just checked, that the peer is fine to be dialed.
    let PeerInfo {
        address: addr, alias, ..
    } = peerlist.0.read().await.info(&peer_id).expect("peer must exist");

    let mut dial_attempt = 0;

    peerlist
        .0
        .write()
        .await
        .update_metrics(&peer_id, |m| {
            m.num_dials += 1;
            dial_attempt = m.num_dials;
        })
        .expect("peer must exist");

    debug!(
        "Dialing peer: {} ({}) attempt: #{}.",
        alias,
        alias!(peer_id),
        dial_attempt
    );

    // TODO: We also use `Swarm::dial_addr` here (instead of `Swarm::dial`) for now. See if it's better to change
    // that.
    Swarm::dial(swarm, addr).map_err(|e| Error::DialingPeerFailed(peer_id, e))?;

    Ok(())
}

fn hang_up(swarm: &mut Swarm<SwarmBehaviour>, peer_id: PeerId) {
    debug!("Hanging up on: {}.", alias!(peer_id));

    let _ = Swarm::disconnect_peer_id(swarm, peer_id);
}
