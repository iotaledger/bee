// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::{
    endpoints::{storage::StorageBackend, ApiArgsFullNode},
    types::{dtos::PeerDto, responses::PeersResponse},
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers", get(peers::<B>))
}

pub(crate) async fn peers<B: StorageBackend>(Extension(args): Extension<Arc<ApiArgsFullNode<B>>>) -> impl IntoResponse {
    let mut peers_dtos = Vec::new();
    for peer in args.peer_manager.get_all() {
        peers_dtos.push(PeerDto::from(peer.as_ref()));
    }
    Json(PeersResponse(peers_dtos))
}
