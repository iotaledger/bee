// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_api_types::{dtos::PeerDto, responses::PeerResponse};
use bee_gossip::PeerId;

use crate::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers/:peer_id", get(peers::<B>))
}

async fn peers<B: StorageBackend>(
    CustomPath(peer_id): CustomPath<String>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<PeerResponse, ApiError> {
    let peer_id = peer_id
        .parse::<PeerId>()
        .map_err(|_| ApiError::BadRequest("invalid peer id"))?;

    args.peer_manager
        .get_map(&peer_id, |peer_entry| {
            Ok(PeerResponse(PeerDto::from(peer_entry.0.as_ref())))
        })
        .unwrap_or(Err(ApiError::NotFound))
}
