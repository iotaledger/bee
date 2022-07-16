// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, http::StatusCode, routing::delete, Router};
use bee_gossip::{Command::RemovePeer, PeerId};
use log::error;

use crate::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers/:peer_id", delete(peers_remove::<B>))
}

async fn peers_remove<B: StorageBackend>(
    CustomPath(peer_id): CustomPath<String>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<StatusCode, ApiError> {
    let peer_id = peer_id
        .parse::<PeerId>()
        .map_err(|_| ApiError::BadRequest("invalid peer id"))?;

    if let Err(e) = args.network_command_sender.send(RemovePeer { peer_id }) {
        error!("cannot remove peer: {}", e);
        return Err(ApiError::InternalServerError);
    }

    Ok(StatusCode::NO_CONTENT)
}
