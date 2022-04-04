// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod error;
pub mod routes;
pub mod storage;

pub mod auth;

use std::{any::TypeId, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::{extractor_middleware, Extension},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_gossip::{Keypair, NetworkCommandSender, PeerId};
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
use config::RestApiConfig;
use log::info;
use storage::StorageBackend;
use tokio::sync::mpsc;

use crate::endpoints::{auth::Auth, error::ApiError, routes::filter_all};

pub(crate) type NetworkId = (String, u64);
pub(crate) type Bech32Hrp = String;
#[cfg(feature = "dashboard")]
pub(crate) type DashboardUsername = String;

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
    pub node_id: PeerId,
    pub node_keypair: Keypair,
    pub rest_api_config: RestApiConfig,
    pub protocol_config: ProtocolConfig,
    pub network_id: NetworkId,
    pub bech32_hrp: Bech32Hrp,
    #[cfg(feature = "dashboard")]
    pub dashboard_username: DashboardUsername,
}

pub struct InitConfigEntryNode {
    pub rest_api_config: RestApiConfig,
}

pub struct ApiArgsFullNode<B: StorageBackend> {
    pub(crate) node_id: PeerId,
    pub(crate) node_keypair: Keypair,
    pub(crate) rest_api_config: RestApiConfig,
    pub(crate) protocol_config: ProtocolConfig,
    pub(crate) network_id: NetworkId,
    pub(crate) bech32_hrp: Bech32Hrp,
    pub(crate) storage: ResourceHandle<B>,
    pub(crate) bus: ResourceHandle<Bus<'static>>,
    pub(crate) node_info: ResourceHandle<NodeInfo>,
    pub(crate) tangle: ResourceHandle<Tangle<B>>,
    pub(crate) peer_manager: ResourceHandle<PeerManager>,
    pub(crate) requested_messages: ResourceHandle<RequestedMessages>,
    pub(crate) network_command_sender: ResourceHandle<NetworkCommandSender>,
    pub(crate) message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    pub(crate) message_requester: MessageRequesterWorker,
    pub(crate) consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
    #[cfg(feature = "dashboard")]
    pub(crate) dashboard_username: DashboardUsername,
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
            node_id: config.node_id,
            node_keypair: config.node_keypair,
            rest_api_config: config.rest_api_config,
            protocol_config: config.protocol_config,
            network_id: config.network_id,
            bech32_hrp: config.bech32_hrp,
            #[cfg(feature = "dashboard")]
            dashboard_username: config.dashboard_username,
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

            let app = Router::new()
                .merge(filter_all::<N::Backend>())
                .route_layer(extractor_middleware::<Auth<N::Backend>>())
                .layer(Extension(args.clone()))
                .fallback(fallback.into_service());

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

            let app = Router::new()
                .route("/health", get(health_handler))
                .fallback(fallback.into_service());

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

async fn fallback() -> impl IntoResponse {
    ApiError::Forbidden
}
