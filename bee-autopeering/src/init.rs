// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command::{Command, CommandTx},
    config::AutopeeringConfig,
    delay::{Cronjob, DelayFactoryBuilder, DelayFactoryMode},
    discovery::manager::{
        DiscoveryManager, DiscoveryManagerConfig, DEFAULT_QUERY_INTERVAL_SECS, DEFAULT_REVERIFY_INTERVAL_SECS,
    },
    discovery::messages::VerificationRequest,
    event::{self, EventRx},
    hash,
    local::{self, Local},
    multiaddr,
    packet::{IncomingPacket, MessageType, OutgoingPacket},
    peer,
    peering::{PeeringManager, PeeringManagerConfig},
    peerstore::{self, InMemoryPeerStore, PeerStore},
    request::{self, RequestManager, EXPIRED_REQUEST_REMOVAL_INTERVAL_CHECK_SECS},
    salt::{Salt, DEFAULT_SALT_LIFETIME_SECS},
    server::{server_chan, IncomingPacketSenders, Server, ServerConfig, ServerSocket, ServerTx},
    service_map::{ServiceMap, AUTOPEERING_SERVICE_NAME},
    task::{ShutdownBus, ShutdownRx, Task},
    time,
};

use std::{
    error,
    future::Future,
    iter,
    net::SocketAddr,
    ops::DerefMut as _,
    time::{Duration, SystemTime},
};

const NUM_TASKS: usize = 9;

/// Initializes the autopeering service.
pub async fn init<S, I, Q>(
    config: AutopeeringConfig,
    version: u32,
    network_name: I,
    local: Local,
    peerstore_config: <S as PeerStore>::Config,
    quit_signal: Q,
) -> Result<EventRx, Box<dyn error::Error>>
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
    log::info!("Network name/id: {}/{}", network_name.as_ref(), network_id);
    log::info!("Protocol_version: {}", version);
    log::info!("Local id: {}", local.peer_id());
    log::info!(
        "Local public key: {}",
        multiaddr::from_pubkey_to_base58(&local.public_key())
    );
    log::info!("Current time: {}", time::datetime_now());
    log::info!(
        "Local private salt expiration time: {}",
        time::datetime(local.private_salt().expect("missing private salt").expiration_time())
    );
    log::info!(
        "Local public salt expiration time: {}",
        time::datetime(local.public_salt().expect("missing private salt").expiration_time())
    );
    log::info!("Bind address: {}", config.bind_addr);

    // Create a bus to distribute the shutdown signal to all spawned tasks. The const generic always matches
    // the number of required looping tasks.
    let (shutdown_bus, mut shutdown_reg) = ShutdownBus::<NUM_TASKS>::new();
    Task::spawn(
        async move {
            quit_signal.await;
            shutdown_bus.trigger();
        },
        "Shutdown-Listener",
    );

    // Create or load a peer store.
    let peerstore = S::new(peerstore_config);

    // Create channels for inbound/outbound communication with the UDP server.
    let (discovery_tx, discovery_rx) = server_chan::<IncomingPacket>();
    let (peering_tx, peering_rx) = server_chan::<IncomingPacket>();
    let incoming_senders = IncomingPacketSenders {
        discovery_tx,
        peering_tx,
    };

    let (event_tx, event_rx) = event::event_chan();

    // Initialize the server managing the UDP socket I/O. It receives a [`Local`] in order to sign outgoing packets.
    let server_config = ServerConfig::new(&config);
    let (server, outgoing_tx) = Server::new(server_config, local.clone(), incoming_senders);
    server.init(&mut shutdown_reg).await;

    // Create a request manager that creates and keeps track of outgoing requests.
    let request_mngr = RequestManager::new(version, network_id, config.bind_addr, local.clone());

    // Spawn the discovery manager handling discovery requests/responses.
    let discovery_config = DiscoveryManagerConfig::new(&config, version, network_id);
    let discovery_socket = ServerSocket::new(discovery_rx, outgoing_tx.clone());
    let discovery_mngr = DiscoveryManager::new(
        discovery_config,
        local.clone(),
        discovery_socket,
        request_mngr.clone(),
        peerstore.clone(),
        event_tx.clone(),
    );
    let command_tx = discovery_mngr.init(&mut shutdown_reg).await;

    // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    let peering_config = PeeringManagerConfig::new(&config, version, network_id);
    let peering_socket = ServerSocket::new(peering_rx, outgoing_tx.clone());
    let peering_mngr = PeeringManager::new(
        peering_config,
        local.clone(),
        peering_socket,
        request_mngr.clone(),
        peerstore.clone(),
        event_tx.clone(),
        command_tx.clone(),
    );
    Task::spawn_runnable(peering_mngr, shutdown_reg.register());

    // Remove unanswered requests regularly.
    Task::spawn(
        Cronjob::<0>::repeat(
            request_mngr.clone(),
            iter::repeat(Duration::from_secs(EXPIRED_REQUEST_REMOVAL_INTERVAL_CHECK_SECS)),
            request::remove_expired_requests_cmd(),
            (),
            shutdown_reg.register(),
        ),
        "Expired-Request-Removal",
    );

    // Update salts regularly.
    Task::spawn(
        Cronjob::repeat(
            local.clone(),
            iter::repeat(Duration::from_secs(DEFAULT_SALT_LIFETIME_SECS)),
            local::update_salts_cmd(),
            event_tx.clone(),
            shutdown_reg.register(),
        ),
        "Salt-Update",
    );

    impl Cronjob<0> for () {
        type Cancel = ShutdownRx;
        type Context = CommandTx;
        type DelayIter = iter::Repeat<Duration>;
    }

    // Send (re-)verfication requests regularly.
    Task::spawn(
        Cronjob::<0>::repeat(
            (),
            iter::repeat(Duration::from_secs(DEFAULT_REVERIFY_INTERVAL_SECS)),
            Box::new(|&(), ctx: &_| {
                ctx.send(Command::SendVerificationRequests)
                    .expect("error sending verification requests command");
            }),
            command_tx.clone(),
            shutdown_reg.register(),
        ),
        "Reverification-Requests",
    );

    // Send discovery requests regularly.
    Task::spawn(
        Cronjob::<0>::repeat(
            (),
            iter::repeat(Duration::from_secs(DEFAULT_QUERY_INTERVAL_SECS)),
            Box::new(|&(), ctx: &_| {
                ctx.send(Command::SendDiscoveryRequests)
                    .expect("error sending discovery requests command");
            }),
            command_tx.clone(),
            shutdown_reg.register(),
        ),
        "Discovery-Requests",
    );

    log::debug!("Autopeering initialized.");

    Ok(event_rx)
}
