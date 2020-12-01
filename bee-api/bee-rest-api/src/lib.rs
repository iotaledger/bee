// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod config;
mod filters;
mod handlers;
pub mod storage;
mod types;

use crate::{
    config::RestApiConfig,
    types::{ErrorBody, ErrorResponse},
};
use async_trait::async_trait;
use bee_common::{
    node::{Node, NodeBuilder},
    worker::{Error as WorkerError, Worker},
};
use bee_protocol::{tangle::MsTangle, MessageSubmitterWorker, TangleWorker};
use log::info;
use std::{any::TypeId, convert::Infallible};
use warp::{http::StatusCode, Filter, Rejection, Reply};

use crate::{filters::CustomRejection, storage::Backend};

pub(crate) type NetworkId = (String, u64);

pub async fn init<N: Node>(config: RestApiConfig, network_id: NetworkId, node_builder: N::Builder) -> N::Builder
where
    N::Backend: Backend,
{
    node_builder.with_worker_cfg::<ApiWorker>((config, network_id))
}
pub struct ApiWorker;
#[async_trait]
impl<N: Node> Worker<N> for ApiWorker
where
    N::Backend: Backend,
{
    type Config = (RestApiConfig, NetworkId);
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<MessageSubmitterWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let rest_config = config.0;
        let network_id = config.1;

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let message_submitter = node.worker::<MessageSubmitterWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let routes = filters::all(tangle, storage, message_submitter, network_id).recover(handle_rejection);

            let (_, server) =
                warp::serve(routes).bind_with_graceful_shutdown(rest_config.binding_socket_addr(), async {
                    shutdown.await.ok();
                });

            server.await;

            info!("Stopped.");
        });

        Ok(Self)
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (http_code, err_code, desc) = match err.find() {
        Some(CustomRejection::NotFound(msg)) => (StatusCode::NOT_FOUND, "404", msg),
        Some(CustomRejection::BadRequest(msg)) => (StatusCode::BAD_REQUEST, "400", msg),
        Some(CustomRejection::ServiceUnavailable(msg)) => (StatusCode::SERVICE_UNAVAILABLE, "503", msg),
        _ => {
            eprintln!("unhandled rejection: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "500", &"internal server error")
        }
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&ErrorResponse::new(ErrorBody {
            code: err_code,
            message: desc,
        })),
        http_code,
    ))
}
