use crate::{
    peers::{PeerInfo, PeerList},
    service::{
        commands::{Command, CommandReceiver},
        events::InternalEventSender,
    },
    swarm,
    swarm::SwarmBehavior,
};

use futures::channel::oneshot;
use libp2p::{identity::Keypair, Multiaddr, Swarm};
use log::*;

pub struct HostConfig {
    pub local_keys: Keypair,
    pub bind_address: Multiaddr,
    pub peerlist: PeerList,
    pub internal_event_sender: InternalEventSender,
    pub internal_command_receiver: CommandReceiver,
    pub shutdown: oneshot::Receiver<()>,
}

pub struct Host {}

impl Host {
    // #[async_trait]
    // impl<N: Node> Worker<N> for Service {
    //     type Config = NetworkServiceConfig;
    //     type Error = Infallible;

    pub async fn start(config: HostConfig) {
        info!("Network host started.");

        let HostConfig {
            local_keys,
            bind_address,
            peerlist,
            internal_event_sender,
            mut internal_command_receiver,
            mut shutdown,
        } = config;

        let local_keys_clone = local_keys.clone();
        let internal_event_sender_clone = internal_event_sender.clone();

        // A swarm for listening.
        let mut swarm = swarm::build_swarm(&local_keys_clone, internal_event_sender_clone)
            .await
            .expect("Fatal error: creating transport layer failed.");

        info!("Binding address(es): {}", bind_address);

        let _ = Swarm::listen_on(&mut swarm, bind_address).expect("Fatal error: address binding failed.");

        info!("Host running.");

        loop {
            // let next_event = Swarm::next_event(&mut swarm);
            let next = Swarm::next(&mut swarm);
            let recv_command = (&mut internal_command_receiver).recv();

            tokio::select! {
                // Break on shutdown signal
                _ = &mut shutdown => break,
                // Process swarm event
                event = next => {
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

        info!("Host shutting down...");
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
