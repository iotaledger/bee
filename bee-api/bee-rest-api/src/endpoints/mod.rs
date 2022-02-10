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

use bee_gossip::NetworkCommandSender;
use bee_ledger::workers::consensus::ConsensusWorker;
use bee_protocol::workers::{
    config::ProtocolConfig, MessageRequesterWorker, MessageSubmitterWorker, PeerManager, PeerManagerResWorker,
    RequestedMessages,
};
use bee_runtime::{
    node::{Node, NodeBuilder},
    worker::{Error as WorkerError, Worker},
};
use bee_tangle::{Tangle, TangleWorker};

use async_trait::async_trait;
use log::{error, info};
use warp::{http::StatusCode, Filter, Rejection, Reply};

use std::{any::TypeId, convert::Infallible};

pub(crate) type NetworkId = (String, u64);
pub(crate) type Bech32Hrp = String;

pub(crate) const CONFIRMED_THRESHOLD: u32 = 5;

pub async fn init_full_node<N: Node>(
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    node_builder: N::Builder,
) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorkerFullNode>((rest_api_config, protocol_config, network_id, bech32_hrp))
}

pub struct ApiWorkerFullNode;

#[async_trait]
impl<N: Node> Worker<N> for ApiWorkerFullNode
where
    N::Backend: StorageBackend,
{
    type Config = (RestApiConfig, ProtocolConfig, NetworkId, Bech32Hrp);
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
        let rest_api_config = config.0;
        let protocol_config = config.1;
        let network_id = config.2;
        let bech32_hrp = config.3;

        let consensus_worker = node.worker::<ConsensusWorker>().unwrap().tx.clone();
        let tangle = node.resource::<Tangle<N::Backend>>();
        let storage = node.storage();
        let message_submitter = node.worker::<MessageSubmitterWorker>().unwrap().tx.clone();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().clone();
        let requested_messages = node.resource::<RequestedMessages>();
        let peer_manager = node.resource::<PeerManager>();
        let network_controller = node.resource::<NetworkCommandSender>();
        let node_info = node.info();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let routes = routes::filter_all(
                rest_api_config.public_routes.clone(),
                rest_api_config.allowed_ips.clone(),
                tangle,
                storage,
                message_submitter,
                network_id,
                bech32_hrp,
                rest_api_config.clone(),
                protocol_config,
                peer_manager,
                network_controller,
                node_info,
                bus,
                message_requester,
                requested_messages,
                consensus_worker,
            )
            .recover(handle_rejection);

            let (_, server) =
                warp::serve(routes).bind_with_graceful_shutdown(rest_api_config.bind_socket_addr(), async {
                    shutdown.await.ok();
                });

            server.await;

            info!("Stopped.");
        });

        Ok(Self)
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
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

pub async fn init_entry_node<N: Node>(rest_api_config: RestApiConfig, node_builder: N::Builder) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorkerEntryNode>(rest_api_config)
}

pub struct ApiWorkerEntryNode;

#[async_trait]
impl<N: Node> Worker<N> for ApiWorkerEntryNode
where
    N::Backend: StorageBackend,
{
    type Config = RestApiConfig;
    type Error = WorkerError;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let health = warp::path("health").map(|| StatusCode::OK).recover(handle_rejection);

            let (_, server) = warp::serve(health).bind_with_graceful_shutdown(config.bind_socket_addr(), async {
                shutdown.await.ok();
            });

            server.await;

            info!("Stopped.");
        });

        Ok(Self)
    }
}
