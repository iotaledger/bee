// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::config::ROUTE_REMOVE_PEER;

use bee_gossip::{Command::RemovePeer, NetworkCommandSender, PeerId};
use bee_runtime::resource::ResourceHandle;

use std::net::IpAddr;

use crate::endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode};
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::delete,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers/:peer_id", delete(remove_peer::<B>))
}

pub(crate) async fn remove_peer<B: StorageBackend>(
    Path(peer_id): Path<String>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let peer_id = peer_id
        .parse::<PeerId>()
        .map_err(|_| ApiError::BadRequest("invalid peer id".to_string()))?;

    if let Err(e) = args.network_command_sender.send(RemovePeer { peer_id }) {
        return Err(ApiError::NotFound(format!("failed to remove peer: {}", e)));
    }
    Ok(StatusCode::NO_CONTENT)
}
