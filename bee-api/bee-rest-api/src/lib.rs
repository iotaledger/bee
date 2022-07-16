// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Bee REST API

// #![deny(missing_docs, warnings)]

#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub mod config;
pub mod error;
pub mod extractors;
pub mod routes;
pub mod storage;

pub mod auth;

use std::{any::TypeId, ops::Deref, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::Extension, handler::Handler, http::StatusCode, middleware::from_extractor, response::IntoResponse,
    routing::get, Router,
};
use bee_gossip::{Keypair, NetworkCommandSender, PeerId};
use bee_ledger::workers::consensus::{ConsensusWorker, ConsensusWorkerCommand};
use bee_protocol::workers::{
    config::ProtocolConfig, BlockRequesterWorker, BlockSubmitterWorker, BlockSubmitterWorkerEvent, PeerManager,
    PeerManagerResWorker, RequestedBlocks,
};
use bee_runtime::{
    event::Bus,
    node::{Node, NodeBuilder, NodeInfo},
    resource::ResourceHandle,
    worker::{Error as WorkerError, Worker},
};
use bee_tangle::{Tangle, TangleWorker};
use log::info;
use tokio::sync::mpsc;

use self::{config::RestApiConfig, storage::StorageBackend};
use crate::{auth::Auth, error::ApiError, routes::filter_all};

pub(crate) const CONFIRMED_THRESHOLD: u32 = 5;

pub fn init_full_node<N: Node>(init_config: InitFullNodeConfig, node_builder: N::Builder) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorkerFullNode>(init_config)
}

pub fn init_entry_node<N: Node>(init_config: InitEntryNodeConfig, node_builder: N::Builder) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorkerEntryNode>(init_config)
}

pub struct InitFullNodeConfig {
    pub node_id: PeerId,
    pub node_keypair: Keypair,
    pub rest_api_config: RestApiConfig,
    pub protocol_config: ProtocolConfig,
    pub network_name: String,
    pub bech32_hrp: String,
    #[cfg(feature = "dashboard")]
    pub dashboard_username: String,
}

pub struct InitEntryNodeConfig {
    pub rest_api_config: RestApiConfig,
}

pub(crate) struct ApiArgsFullNode<B: StorageBackend>(Arc<ApiArgsFullNodeInner<B>>);

impl<B: StorageBackend> Deref for ApiArgsFullNode<B> {
    type Target = Arc<ApiArgsFullNodeInner<B>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B: StorageBackend> Clone for ApiArgsFullNode<B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct ApiArgsFullNodeInner<B: StorageBackend> {
    pub(crate) node_id: PeerId,
    pub(crate) node_keypair: Keypair,
    pub(crate) rest_api_config: RestApiConfig,
    pub(crate) protocol_config: ProtocolConfig,
    pub(crate) network_name: String,
    pub(crate) bech32_hrp: String,
    pub(crate) storage: ResourceHandle<B>,
    pub(crate) bus: ResourceHandle<Bus<'static>>,
    pub(crate) node_info: ResourceHandle<NodeInfo>,
    pub(crate) tangle: ResourceHandle<Tangle<B>>,
    pub(crate) peer_manager: ResourceHandle<PeerManager>,
    pub(crate) requested_blocks: ResourceHandle<RequestedBlocks>,
    pub(crate) network_command_sender: ResourceHandle<NetworkCommandSender>,
    pub(crate) block_submitter: mpsc::UnboundedSender<BlockSubmitterWorkerEvent>,
    pub(crate) block_requester: BlockRequesterWorker,
    pub(crate) consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
    #[cfg(feature = "dashboard")]
    pub(crate) dashboard_username: String,
}

pub struct ApiWorkerFullNode;

#[async_trait]
impl<N: Node> Worker<N> for ApiWorkerFullNode
where
    N::Backend: StorageBackend,
{
    type Config = InitFullNodeConfig;
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<BlockSubmitterWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let args = ApiArgsFullNode(Arc::new(ApiArgsFullNodeInner {
            node_id: config.node_id,
            node_keypair: config.node_keypair,
            rest_api_config: config.rest_api_config,
            protocol_config: config.protocol_config,
            network_name: config.network_name,
            bech32_hrp: config.bech32_hrp,
            storage: node.storage(),
            bus: node.bus(),
            node_info: node.info(),
            tangle: node.resource::<Tangle<N::Backend>>(),
            peer_manager: node.resource::<PeerManager>(),
            requested_blocks: node.resource::<RequestedBlocks>(),
            network_command_sender: node.resource::<NetworkCommandSender>(),
            block_submitter: node.worker::<BlockSubmitterWorker>().unwrap().tx.clone(),
            block_requester: node.worker::<BlockRequesterWorker>().unwrap().clone(),
            consensus_worker: node.worker::<ConsensusWorker>().unwrap().tx.clone(),
            #[cfg(feature = "dashboard")]
            dashboard_username: config.dashboard_username,
        }));

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let app = Router::new()
                .merge(filter_all::<N::Backend>())
                .route_layer(from_extractor::<Auth<N::Backend>>())
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
    type Config = InitEntryNodeConfig;
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
