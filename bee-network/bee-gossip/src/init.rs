// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Gossip layer initialization.

#![cfg(feature = "full")]

use crate::{
    config::GossipLayerConfig,
    gossip::layer,
    manager::{
        self, GossipManagerCommand, GossipManagerCommandTx as CommandTx, GossipManagerConfig, GossipManagerEvent,
        GossipManagerEventRx as EventRx,
    },
    peer::{
        peer_id::PeerId,
        peer_state_checker::{self, StateCheckContext, STATE_CHECK_INITIAL, STATE_CHECK_INTERVAL},
        peer_state_map::{PeerStateMap, PeerStateMapConfig},
    },
    server::{self, GossipServerCommand, GossipServerConfig, GossipServerEvent},
    task::{self, TaskManager, MAX_SHUTDOWN_PRIORITY},
    Ed25519Keypair,
};

use libp2p_core::identity;

use std::{future::Future, iter};

/// Errors during initialization.
#[derive(Debug, thiserror::Error)]
pub enum BootError {
    /// Initializing the gossip layer failed.
    #[error("Failed to initialize the gossip layer.")]
    InitGossipLayer,

    /// Binding the server to its configured listening address failed.
    #[error("Failed to bind the gossip server to the configured listen address.")]
    BindGossipServer,
}

/// Initializes the gossip layer in standalone mode.
pub async fn init(
    config: GossipLayerConfig,
    local_keys: Ed25519Keypair,
    network_id: u64,
    term_signal: impl Future + Send + Unpin + 'static,
) -> Result<(CommandTx, EventRx), Box<dyn std::error::Error>> {
    use crate::{manager::GossipManager, server::GossipServer};

    let (server_config, manager_config, command_tx, event_rx, peer_state_map) =
        common_init(config, local_keys, network_id).await?;

    let mut task_mngr = TaskManager::new();

    // Check on the peers regularly.
    let ctx = StateCheckContext { peer_state_map };
    let f = peer_state_checker::check_peer_states_fn();
    let delay = iter::once(STATE_CHECK_INITIAL).chain(iter::repeat(STATE_CHECK_INTERVAL));
    task_mngr.repeat(f, delay, ctx, "Peer state checker", MAX_SHUTDOWN_PRIORITY);

    let (mngr_shutdown_signal_tx, mngr_shutdown_signal_rx) = task::shutdown_chan();
    let (serv_shutdown_signal_tx, serv_shutdown_signal_rx) = task::shutdown_chan();

    tokio::spawn(async move {
        term_signal.await;

        task_mngr.shutdown().await.expect("task manager shutdown");

        // Panic:
        // Sending the shutdown signal must not fail.
        mngr_shutdown_signal_tx.send(()).expect("gossip manager shutdown send");
        serv_shutdown_signal_tx.send(()).expect("gossip server shutdown send");
    });

    GossipManager::new(mngr_shutdown_signal_rx).start(manager_config).await;
    GossipServer::new(serv_shutdown_signal_rx).start(server_config).await;

    Ok((command_tx, event_rx))
}

/// Contains the `init` function for the use with `bee-runtime` node workers.
pub mod workers {
    use crate::peer::peer_state_checker::workers::{PeerStateChecker, PeerStateCheckerConfig};

    use super::*;
    use bee_runtime::node::{Node, NodeBuilder};

    /// Initializes the gossip layer using `bee-runtime` node workers.
    pub async fn init<N: Node>(
        config: GossipLayerConfig,
        local_keys: Ed25519Keypair,
        network_id: u64,
        mut node_builder: N::Builder,
    ) -> Result<(N::Builder, EventRx), BootError> {
        use crate::{manager::workers::GossipManager, server::workers::GossipServer};

        let (server_config, manager_config, command_tx, event_rx, peer_state_map) =
            common_init(config, local_keys, network_id).await?;

        let peer_state_checker_config = PeerStateCheckerConfig { peer_state_map };

        node_builder = node_builder
            .with_worker_cfg::<GossipServer>(server_config)
            .with_worker_cfg::<GossipManager>(manager_config)
            .with_worker_cfg::<PeerStateChecker>(peer_state_checker_config)
            .with_resource(command_tx);

        Ok((node_builder, event_rx))
    }
}

async fn common_init(
    config: GossipLayerConfig,
    local_keys: Ed25519Keypair,
    network_id: u64,
) -> Result<
    (
        GossipServerConfig,
        GossipManagerConfig,
        CommandTx,
        EventRx,
        PeerStateMap,
    ),
    BootError,
> {
    let local_keys = identity::Keypair::Ed25519(local_keys);
    let local_peer_id: PeerId = local_keys.public().to_peer_id().into();

    // Access config data.
    let GossipLayerConfig {
        bind_addr,
        reconnect_interval: _,
        max_unknown_peers,
        max_discovered_peers,
        manual_peers,
    } = config;

    // Create gossip manager channels.
    let (manager_command_tx, manager_command_rx) = manager::chan::<GossipManagerCommand>();
    let (manager_event_tx, manager_event_rx) = manager::chan::<GossipManagerEvent>();

    // Create gossip server channels.
    let (server_command_tx, server_command_rx) = server::chan::<GossipServerCommand>();
    let (server_event_tx, server_event_rx) = server::chan::<GossipServerEvent>();

    // Initialize the peer state map, which allows us to keep track of each peer's connection state.
    let peer_state_map_config = PeerStateMapConfig {
        local_peer_id,
        max_unknown_peers,
        max_discovered_peers,
    };
    let peer_state_map = PeerStateMap::new(peer_state_map_config, manual_peers);

    log::debug!("Added {} manual peers.", peer_state_map.len());

    // Initialize the gossip layer, i.e. set up the transport layer and start running all provided protocols on top of
    // it.
    let gossip_layer =
        layer::init_gossip_layer(local_keys, local_peer_id, network_id).map_err(|_| BootError::InitGossipLayer)?;

    // Gossip server configuration.
    let gossip_server_config = GossipServerConfig {
        bind_addr,
        gossip_layer,
        peer_state_map: peer_state_map.clone(),
        server_event_tx,
        server_command_rx,
    };

    // Gossip manager configuration.
    let gossip_manager_config = GossipManagerConfig {
        senders: manager::Senders {
            manager_event_tx,
            server_command_tx,
        },
        receivers: manager::Receivers {
            manager_command_rx,
            server_event_rx,
        },
        peer_state_map: peer_state_map.clone(),
    };

    Ok((
        gossip_server_config,
        gossip_manager_config,
        manager_command_tx,
        manager_event_rx,
        peer_state_map,
    ))
}
