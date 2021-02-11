use crate::{
    peers::{BannedAddrList, BannedPeerList, PeerInfo, PeerList},
    service::{
        commands::{Command, CommandReceiver},
        events::InternalEventSender,
        Service,
    },
    swarm,
    swarm::SwarmBehavior,
};

use bee_runtime::{node::Node, worker::Worker};

use async_trait::async_trait;
use libp2p::{identity::Keypair, Multiaddr, Swarm};
use log::*;

use std::{any::TypeId, convert::Infallible};

pub struct HostConfig {
    pub local_keys: Keypair,
    pub bind_address: Multiaddr,
    pub peerlist: PeerList,
    pub banned_addrlist: BannedAddrList,
    pub banned_peerlist: BannedPeerList,
    pub internal_event_sender: InternalEventSender,
    pub internal_command_receiver: CommandReceiver,
}

#[derive(Default)]
pub struct Host {}

#[async_trait]
impl<N: Node> Worker<N> for Host {
    type Config = HostConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<Service>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let HostConfig {
            local_keys,
            bind_address,
            peerlist,
            banned_addrlist,
            banned_peerlist,
            internal_event_sender,
            mut internal_command_receiver,
        } = config;

        node.spawn::<Self, _, _>(|mut shutdown| async move {
            let local_keys_clone = local_keys.clone();
            let internal_event_sender_clone = internal_event_sender.clone();

            // A swarm for listening.
            let mut swarm = swarm::build_swarm(&local_keys_clone, internal_event_sender_clone)
                .await
                .expect("Fatal error: creating transport layer failed.");

            info!("Binding address(es): {}", bind_address);

            let _ = Swarm::listen_on(&mut swarm, bind_address).expect("Fatal error: address binding failed.");

            info!("Host listening for incoming connections.");

            loop {
                let swarm_next = Swarm::next(&mut swarm);
                let recv_command = (&mut internal_command_receiver).recv();

                tokio::select! {
                    // Break on shutdown signal
                    _ = &mut shutdown => break,
                    // Process swarm event
                    event = swarm_next => {
                        // All events are handled by the `NetworkBehaviourEventProcess`es.
                        // I.e. the `swarm.next()` future drives the `Swarm` without ever
                        // terminating.
                        unreachable!("Unexpected event: {:?}", event);
                    }
                    // Process command
                    command = recv_command => {
                        if let Some(command) = command {
                            process_command(command, &mut swarm, &peerlist).await;
                        }
                    },
                }
            }

            info!("Listener stopped.");
        });

        info!("Network host started.");

        Ok(Self::default())
    }
}

async fn process_command(command: Command, mut swarm: &mut Swarm<SwarmBehavior>, peerlist: &PeerList) {
    match command {
        // process_command(&mut swarm, command, &peerlist).await;
        Command::DialPeer { peer_id } => {
            info!("Dialing peer {}.", peer_id);

            if let Ok(info) = peerlist.get_info(&peer_id).await {
                let PeerInfo { address, .. } = info;
                let _ = Swarm::dial_addr(&mut swarm, address);
            }
        }
        Command::DialAddress { address } => {
            info!("Dialing address {}.", address);
            let _ = Swarm::dial_addr(&mut swarm, address);
        }
        Command::DiscoverPeers => {
            // TODO
        }
        _ => {}
    }
}
