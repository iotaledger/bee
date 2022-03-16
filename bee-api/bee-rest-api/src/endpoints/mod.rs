// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
pub mod config;
// pub mod permission;
pub mod error;
pub mod routes;
pub mod storage;

use config::RestApiConfig;
use storage::StorageBackend;

use bee_gossip::NetworkCommandSender;
use bee_ledger::workers::consensus::{ConsensusWorker, ConsensusWorkerCommand};
use bee_protocol::workers::{
    config::ProtocolConfig, MessageRequesterWorker, MessageSubmitterWorker, MessageSubmitterWorkerEvent, PeerManager,
    PeerManagerResWorker, RequestedMessages,
};
use bee_runtime::{
    event::Bus,
    node::{Node, NodeBuilder, NodeInfo},
    resource::ResourceHandle,
    worker::{Error as WorkerError, Worker},
};
use bee_tangle::{Tangle, TangleWorker};

use async_trait::async_trait;
use axum::{extract::Extension, http::StatusCode, routing::get, Router};

use log::info;
use tokio::sync::mpsc;

use crate::endpoints::routes::filter_all;
use std::{any::TypeId, sync::Arc};

pub(crate) type NetworkId = (String, u64);
pub(crate) type Bech32Hrp = String;

pub(crate) const CONFIRMED_THRESHOLD: u32 = 5;

pub async fn init_full_node<N: Node>(init_config: InitConfigFullNode, node_builder: N::Builder) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorkerFullNode>(init_config)
}

pub async fn init_entry_node<N: Node>(init_config: InitConfigEntryNode, node_builder: N::Builder) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorkerEntryNode>(init_config)
}

pub struct InitConfigFullNode {
    pub rest_api_config: RestApiConfig,
    pub protocol_config: ProtocolConfig,
    pub network_id: NetworkId,
    pub bech32_hrp: Bech32Hrp,
}

pub struct InitConfigEntryNode {
    pub rest_api_config: RestApiConfig,
}

pub struct ApiArgsFullNode<B: StorageBackend> {
    pub rest_api_config: RestApiConfig,
    pub protocol_config: ProtocolConfig,
    pub network_id: NetworkId,
    pub bech32_hrp: Bech32Hrp,
    pub storage: ResourceHandle<B>,
    pub bus: ResourceHandle<Bus<'static>>,
    pub node_info: ResourceHandle<NodeInfo>,
    pub tangle: ResourceHandle<Tangle<B>>,
    pub peer_manager: ResourceHandle<PeerManager>,
    pub requested_messages: ResourceHandle<RequestedMessages>,
    pub network_command_sender: ResourceHandle<NetworkCommandSender>,
    pub message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    pub message_requester: MessageRequesterWorker,
    pub consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
}

pub struct ApiWorkerFullNode;

#[async_trait]
impl<N: Node> Worker<N> for ApiWorkerFullNode
where
    N::Backend: StorageBackend,
{
    type Config = InitConfigFullNode;
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MessageSubmitterWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let args = Arc::new(ApiArgsFullNode {
            rest_api_config: config.rest_api_config,
            protocol_config: config.protocol_config,
            network_id: config.network_id,
            bech32_hrp: config.bech32_hrp,
            storage: node.storage(),
            bus: node.bus(),
            node_info: node.info(),
            tangle: node.resource::<Tangle<N::Backend>>(),
            peer_manager: node.resource::<PeerManager>(),
            requested_messages: node.resource::<RequestedMessages>(),
            network_command_sender: node.resource::<NetworkCommandSender>(),
            message_submitter: node.worker::<MessageSubmitterWorker>().unwrap().tx.clone(),
            message_requester: node.worker::<MessageRequesterWorker>().unwrap().clone(),
            consensus_worker: node.worker::<ConsensusWorker>().unwrap().tx.clone(),
        });

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            // let routes = routes::filter_all(
            // )
            // .recover(handle_rejection);

            let app = Router::new()
                .merge(filter_all::<N::Backend>())
                .layer(Extension(args.clone()));

            axum::Server::bind(&args.rest_api_config.bind_socket_addr())
                .serve(app.into_make_service())
                .with_graceful_shutdown(async {
                    shutdown.await.ok();
                })
                .await
                .unwrap(); // TODO: handle unwrap

            info!("Stopped.");
        });

        Ok(Self)
    }
}

pub struct ApiWorkerEntryNode;

#[async_trait]
impl<N: Node> Worker<N> for ApiWorkerEntryNode
where
    N::Backend: StorageBackend,
{
    type Config = InitConfigEntryNode;
    type Error = WorkerError;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            async fn health_handler() -> StatusCode {
                StatusCode::OK
            }

            let app = Router::new().route("/health", get(health_handler));

            axum::Server::bind(&config.rest_api_config.bind_socket_addr())
                .serve(app.into_make_service())
                .with_graceful_shutdown(async {
                    shutdown.await.ok();
                })
                .await
                .unwrap(); // TODO: handle unwrap

            info!("Stopped.");
        });

        Ok(Self)
    }
}
