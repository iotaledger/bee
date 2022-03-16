// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_PEER,  path_params::peer_id, permission::has_permission,
        rejection::CustomRejection,
    },
    types::{dtos::PeerDto, responses::PeerResponse},
};

use bee_gossip::PeerId;
use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;


use std::net::IpAddr;


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
        .route("/peers/:peer_id", get(peer::<B>))
}

pub(crate) async fn peer<B: StorageBackend>(
    Path(peer_id): Path<String>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let peer_id = peer_id
        .parse::<PeerId>()
        .map_err(|_| ApiError::BadRequest("invalid peer id".to_string()))?;

    args.peer_manager
        .get_map(&peer_id, |peer_entry| {
            Ok(Json(PeerResponse(PeerDto::from(peer_entry.0.as_ref()))))
        })
        .unwrap_or_else(|| Err(ApiError::NotFound("peer not found".to_string())))
}
