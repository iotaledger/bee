// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_gossip::PeerId;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::{dtos::PeerDto, responses::PeerResponse},
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers/:peer_id", get(peer::<B>))
}

pub(crate) async fn peer<B: StorageBackend>(
    Path(peer_id): Path<String>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
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
