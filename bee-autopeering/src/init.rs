// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    delay::{DelayFactoryBuilder, DelayFactoryMode, DelayedRepeat},
    discovery::{DiscoveryEventRx, DiscoveryManager, DiscoveryManagerConfig},
    discovery_messages::VerificationRequest,
    hash,
    local::Local,
    packet::{IncomingPacket, MessageType, OutgoingPacket},
    peering::{PeeringEventRx, PeeringManager, PeeringManagerConfig},
    peerstore::{InMemoryPeerStore, PeerStore},
    request::RequestManager,
    salt::{Salt, DEFAULT_SALT_LIFETIME},
    server::{server_chan, IncomingPacketSenders, Server, ServerConfig, ServerSocket, ServerTx},
    service_map::{ServiceMap, AUTOPEERING_SERVICE_NAME},
    shutdown::ShutdownBus,
    time,
};

use std::{error, future::Future, net::SocketAddr, ops::DerefMut as _, time::SystemTime};

/// Initializes the autopeering service.
pub async fn init<S, I, Q>(
    config: AutopeeringConfig,
    version: u32,
    network_name: I,
    local: Local,
    peerstore_config: <S as PeerStore>::Config,
    quit_signal: Q,
) -> Result<(DiscoveryEventRx, PeeringEventRx), Box<dyn error::Error>>
where
    S: PeerStore + 'static,
    I: AsRef<str>,
    Q: Future + Send + 'static,
{
    let network_id = hash::fnv32(&network_name);

    log::info!("---------------------------------------------------------------------------------------------------");
    log::info!("WARNING:");
    log::info!("The autopeering system will disclose your public IP address to possibly all nodes and entry points.");
    log::info!("Please disable it if you do not want this to happen!");
    log::info!("---------------------------------------------------------------------------------------------------");
    log::info!("network_name/id: {}/{}", network_name.as_ref(), network_id);
    log::info!("protocol_version: {}", version);
    // TODO: log the salt expiration time

    // Create a bus to distribute the shutdown signal to all spawned tasks. The const generic always matches
    // the number of required permanent tasks.
    let (shutdown_bus, mut shutdown_reg) = ShutdownBus::<7>::new();
    tokio::spawn(async move {
        quit_signal.await;
        shutdown_bus.trigger();
    });

    // Create or load a peer store.
    let peerstore = S::new(peerstore_config);

    // Create channels for inbound/outbound communication with the UDP server.
    let (discovery_tx, discovery_rx) = server_chan::<IncomingPacket>();
    let (peering_tx, peering_rx) = server_chan::<IncomingPacket>();
    let incoming_senders = IncomingPacketSenders {
        discovery_tx,
        peering_tx,
    };

    // Spawn the server managing the UDP socket I/O. It receives a [`Local`] in order to sign outgoing packets.
    let server_config = ServerConfig::new(&config);
    let (server, outgoing_tx) = Server::new(
        server_config,
        local.clone(),
        incoming_senders,
        shutdown_reg.register(),
        shutdown_reg.register(),
    );
    tokio::spawn(server.run());

    // Create a request manager that creates and keeps track of outgoing requests.
    let request_mngr = RequestManager::new(version, network_id, config.bind_addr, local.clone());

    // Spawn a cronjob that regularly removes unanswered requests.
    let remove_expired_requests = Box::new(|mngr: &RequestManager, ctx: &_| {
        let now = time::unix_now_secs();
        let mut guard = mngr.open_requests.write().expect("error getting write access");
        let requests = guard.deref_mut();
        // Retain only those, which expire in the future.
        requests.retain(|_, v| v.expiration_time > now);
        if !requests.is_empty() {
            log::debug!("Open requests: {}", requests.len());
        }
    });
    tokio::spawn(DelayedRepeat::<0>::repeat(
        request_mngr.clone(),
        DelayFactoryBuilder::new(DelayFactoryMode::Constant(1000)).finish(),
        remove_expired_requests,
        (),
        shutdown_reg.register(),
    ));

    // Regularly update the salts of the local peer.
    let update_salts = Box::new(|local: &Local, ctx: &_| {
        local.set_private_salt(Salt::default());
        local.set_public_salt(Salt::default());
        log::info!("Salts updated");
        // TODO: publish `SaltUpdated` event
    });
    tokio::spawn(DelayedRepeat::repeat(
        local.clone(),
        DelayFactoryBuilder::new(DelayFactoryMode::Constant(DEFAULT_SALT_LIFETIME.as_millis() as u64)).finish(),
        update_salts,
        (),
        shutdown_reg.register(),
    ));

    // Spawn the discovery manager handling discovery requests/responses.
    let discovery_config = DiscoveryManagerConfig::new(&config, version, network_id);
    let discovery_socket = ServerSocket::new(discovery_rx, outgoing_tx.clone());
    let (discovery_mngr, discovery_event_rx) = DiscoveryManager::new(
        discovery_config,
        local.clone(),
        discovery_socket,
        request_mngr.clone(),
        peerstore.clone(),
        shutdown_reg.register(),
    );
    tokio::spawn(discovery_mngr.run());

    // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    let peering_config = PeeringManagerConfig::new(&config, version, network_id);
    let peering_socket = ServerSocket::new(peering_rx, outgoing_tx.clone());
    let (peering_mngr, peering_event_rx) = PeeringManager::new(
        peering_config,
        local.clone(),
        peering_socket,
        request_mngr.clone(),
        peerstore.clone(),
        shutdown_reg.register(),
    );
    tokio::spawn(peering_mngr.run());

    // Send regular (re-) verification requests.
    let send_verification_requests = Box::new(|peerstore: &S, ctx: &(RequestManager, ServerTx)| {
        log::debug!("Sending verification requests to peers.");

        let request_mngr = &ctx.0;
        let server_tx = &ctx.1;

        for peer in peerstore.peers() {
            let peer_id = peer.peer_id();
            let target_addr = peer.ip_address();

            let verif_req = request_mngr.new_verification_request(peer_id, target_addr);
            let msg_bytes = verif_req
                .to_protobuf()
                .expect("error encoding verification request")
                .to_vec();

            let port = peer
                .services()
                .get(AUTOPEERING_SERVICE_NAME)
                .expect("missing autopeering service")
                .port();

            server_tx.send(OutgoingPacket {
                msg_type: MessageType::VerificationRequest,
                msg_bytes,
                target_socket_addr: SocketAddr::new(target_addr, port),
            });
        }
    });
    tokio::spawn(DelayedRepeat::<0>::repeat(
        peerstore.clone(),
        // DelayFactoryBuilder::new(DelayFactoryMode::Constant(10 * 60 * 1000)).finish(),
        DelayFactoryBuilder::new(DelayFactoryMode::Constant(60 * 1000)).finish(),
        send_verification_requests,
        (request_mngr.clone(), outgoing_tx.clone()),
        shutdown_reg.register(),
    ));

    // TODO: Send discovery requests to all verified peers.

    log::debug!("Autopeering initialized.");

    Ok((discovery_event_rx, peering_event_rx))
}
