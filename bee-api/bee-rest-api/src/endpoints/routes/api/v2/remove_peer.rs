// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::delete,
    Router,
};
use bee_gossip::{Command::RemovePeer, PeerId};

use crate::endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers/:peer_id", delete(remove_peer::<B>))
}

pub(crate) async fn remove_peer<B: StorageBackend>(
    Path(peer_id): Path<String>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let peer_id = peer_id
        .parse::<PeerId>()
        .map_err(|_| ApiError::BadRequest("invalid peer id".to_string()))?;

    if let Err(e) = args.network_command_sender.send(RemovePeer { peer_id }) {
        return Err(ApiError::NotFound(format!("failed to remove peer: {}", e)));
    }
    Ok(StatusCode::NO_CONTENT)
}
