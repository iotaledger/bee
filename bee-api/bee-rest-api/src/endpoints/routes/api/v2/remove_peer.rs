// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_REMOVE_PEER,  path_params::peer_id, permission::has_permission,
    rejection::CustomRejection,
};

use bee_gossip::{Command::RemovePeer, NetworkCommandSender, PeerId};
use bee_runtime::resource::ResourceHandle;

use warp::{filters::BoxedFilter, http::StatusCode, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

use axum::extract::Extension;
use crate::endpoints::ApiArgsFullNode;
use axum::extract::Json;
use axum::Router;
use axum::routing::delete;
use axum::response::IntoResponse;
use crate::endpoints::error::ApiError;
use std::sync::Arc;
use axum::extract::Path;
use crate::endpoints::storage::StorageBackend;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .route("/peers/:peer_id", delete(remove_peer::<B>))
}

pub(crate) async fn remove_peer<B: StorageBackend>(
    Path(peer_id): Path<String>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let peer_id = peer_id
        .parse::<PeerId>()
        .map_err(|_| ApiError::BadRequest("invalid peer id".to_string()))?;

    if let Err(e) = args.network_command_sender.send(RemovePeer { peer_id }) {
        return Err(ApiError::NotFound(format!(
            "failed to remove peer: {}",
            e
        )));
    }
    Ok(StatusCode::NO_CONTENT)
}
