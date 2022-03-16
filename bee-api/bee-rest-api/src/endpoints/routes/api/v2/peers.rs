// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{config::ROUTE_PEERS,  permission::has_permission},
    types::{dtos::PeerDto, responses::PeersResponse},
};

use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::{convert::Infallible, net::IpAddr};

use axum::extract::Extension;
use crate::endpoints::ApiArgsFullNode;
use axum::extract::Json;
use axum::Router;
use axum::routing::get;
use axum::response::IntoResponse;
use crate::endpoints::error::ApiError;
use std::sync::Arc;
use axum::extract::Path;
use crate::endpoints::storage::StorageBackend;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .route("/peers", get(peers::<B>))
}

pub(crate) async fn peers<B: StorageBackend>(Extension(args): Extension<Arc<ApiArgsFullNode<B>>>) -> impl IntoResponse {
    let mut peers_dtos = Vec::new();
    for peer in args.peer_manager.get_all() {
        peers_dtos.push(PeerDto::from(peer.as_ref()));
    }
    Json(PeersResponse(peers_dtos))
}
