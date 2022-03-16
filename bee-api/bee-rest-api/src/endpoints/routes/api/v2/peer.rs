// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{dtos::PeerDto, responses::PeerResponse};

use bee_gossip::PeerId;

use crate::endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode};
use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers/:peer_id", get(peer::<B>))
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
