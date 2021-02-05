// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod constants;
mod filters;

pub mod config;
pub mod handlers;
pub mod storage;
pub mod types;

use crate::{
    config::RestApiConfig,
    filters::CustomRejection,
    handlers::{DefaultErrorResponse, ErrorBody},
    storage::StorageBackend,
};

use bee_network::NetworkController;
use bee_protocol::{config::ProtocolConfig, MessageSubmitterWorker, PeerManager, PeerManagerResWorker};
use bee_runtime::{
    node::{Node, NodeBuilder},
    worker::{Error as WorkerError, Worker},
};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use log::{error, info};
use warp::{http::StatusCode, Filter, Rejection, Reply};

use std::{any::TypeId, convert::Infallible};

pub(crate) type NetworkId = (String, u64);
pub(crate) type Bech32Hrp = String;

pub async fn init<N: Node>(
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    node_builder: N::Builder,
) -> N::Builder
where
    N::Backend: StorageBackend,
{
    node_builder.with_worker_cfg::<ApiWorker>((rest_api_config, protocol_config, network_id, bech32_hrp))
}

pub struct ApiWorker;
#[async_trait]
impl<N: Node> Worker<N> for ApiWorker
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

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let message_submitter = node.worker::<MessageSubmitterWorker>().unwrap().tx.clone();
        let peer_manager = node.resource::<PeerManager>();
        let network_controller = node.resource::<NetworkController>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let routes = filters::all(
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
            )
            .recover(handle_rejection);

            let (_, server) =
                warp::serve(routes).bind_with_graceful_shutdown(rest_api_config.binding_socket_addr(), async {
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
        Some(CustomRejection::Forbidden) => (StatusCode::FORBIDDEN, "403".to_string(), "access forbidden".to_string()),
        Some(CustomRejection::NotFound(reason)) => (StatusCode::NOT_FOUND, "404".to_string(), reason.to_owned()),
        Some(CustomRejection::BadRequest(reason)) => (StatusCode::BAD_REQUEST, "400".to_string(), reason.to_owned()),
        Some(CustomRejection::ServiceUnavailable(reason)) => {
            (StatusCode::SERVICE_UNAVAILABLE, "503".to_string(), reason.to_owned())
        }
        // handle default rejections
        _ => {
            if err.is_not_found() {
                (StatusCode::NOT_FOUND, "404".to_string(), "data not found".to_string())
            } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
                (StatusCode::FORBIDDEN, "403".to_string(), "access forbidden".to_string())
            } else {
                error!("unhandled rejection: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "500".to_string(),
                    "internal server error".to_string(),
                )
            }
        }
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&ErrorBody::new(DefaultErrorResponse {
            code: err_code,
            message: reason,
        })),
        http_code,
    ))
}
