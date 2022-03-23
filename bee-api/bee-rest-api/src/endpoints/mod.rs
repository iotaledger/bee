// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod filters;

pub mod config;
pub mod path_params;
pub mod permission;
pub mod rejection;
pub mod routes;
pub mod storage;

use config::RestApiConfig;
use rejection::CustomRejection;
use storage::StorageBackend;

use crate::types::body::{DefaultErrorResponse, ErrorBody};

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

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc;
use warp::{http::StatusCode, Filter, Rejection, Reply};

use std::{any::TypeId, convert::Infallible, ops::Deref, sync::Arc};

pub(crate) type NetworkId = (String, u64);
pub(crate) type Bech32Hrp = String;
#[cfg(feature = "dashboard")]
pub(crate) type DashboardUsername = String;

pub(crate) const CONFIRMED_THRESHOLD: u32 = 5;

pub fn init_full_node<N: Node>(init_config: InitConfigFullNode, node_builder: N::Builder) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorkerFullNode>(init_config)
}

pub fn init_entry_node<N: Node>(init_config: InitConfigEntryNode, node_builder: N::Builder) -> N::Builder
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

pub(crate) struct ApiArgsFullNodeInner<B: StorageBackend> {
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

    #[cfg_attr(feature = "trace", trace_tools::observe)]
    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let args = ApiArgsFullNode(Arc::new(ApiArgsFullNodeInner {
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
        }));

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let routes = routes::filter_all(args.clone()).recover(|err| async { handle_rejection(err) });

            let (_, server) =
                warp::serve(routes).bind_with_graceful_shutdown(args.rest_api_config.bind_socket_addr(), async {
                    shutdown.await.ok();
                });

            server.await;

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

            let health = warp::path("health").map(|| StatusCode::OK).recover(handle_rejection);

            let (_, server) =
                warp::serve(health).bind_with_graceful_shutdown(config.rest_api_config.bind_socket_addr(), async {
                    shutdown.await.ok();
                });

            server.await;

            info!("Stopped.");
        });

        Ok(Self)
    }
}

fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (http_code, err_code, reason) = match err.find() {
        // handle custom rejections
        Some(CustomRejection::Forbidden) => (StatusCode::FORBIDDEN, "403", "access forbidden"),
        Some(CustomRejection::NotFound(reason)) => (StatusCode::NOT_FOUND, "404", reason.as_str()),
        Some(CustomRejection::BadRequest(reason)) => (StatusCode::BAD_REQUEST, "400", reason.as_str()),
        Some(CustomRejection::ServiceUnavailable(reason)) => (StatusCode::SERVICE_UNAVAILABLE, "503", reason.as_str()),
        // handle default rejections
        _ => {
            if err.is_not_found() {
                (StatusCode::NOT_FOUND, "404", "data not found")
            } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
                (StatusCode::FORBIDDEN, "403", "access forbidden")
            } else {
                error!("unhandled rejection: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "500", "internal server error")
            }
        }
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&ErrorBody::new(DefaultErrorResponse {
            code: err_code.to_string(),
            message: reason.to_string(),
        })),
        http_code,
    ))
}
