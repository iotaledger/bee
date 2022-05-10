// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
    Router::new().route("/peers", get(peers_all::<B>))
}

async fn peers_all<B: StorageBackend>(Extension(args): Extension<ApiArgsFullNode<B>>) -> impl IntoResponse {
    let mut peers = Vec::new();

    for peer in args.peer_manager.get_all() {
        peers.push(PeerDto::from(peer.as_ref()));
    }

    Json(PeersResponse(peers))
}
