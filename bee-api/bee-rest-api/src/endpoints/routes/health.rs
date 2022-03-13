// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_HEALTH,
    permission::has_permission,
    storage::StorageBackend,
};

use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

use std::{
    convert::Infallible,
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};
use axum::{
    routing::{get, post},
    Router,
};
use crate::endpoints::ApiArgsFullNode;
use axum::extract::Extension;
use std::sync::Arc;
use axum::response::IntoResponse;

const HEALTH_CONFIRMED_THRESHOLD: u32 = 2; // in milestones
const HEALTH_MILESTONE_AGE_MAX: u64 = 5 * 60; // in seconds


pub(crate) fn filter<B: StorageBackend>() -> Router {

    Router::new()
        .route("/health", get(health::<B>))


        // .and(has_permission(ROUTE_HEALTH, public_routes, allowed_ips))

}

pub(crate) async fn health<B: StorageBackend>(
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, Infallible> {
    if is_healthy(&args.tangle, &args.peer_manager).await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub async fn is_healthy<B: StorageBackend>(
    tangle: &Tangle<B>,
    peer_manager: &PeerManager,
) -> bool {
    if !tangle.is_confirmed_threshold(HEALTH_CONFIRMED_THRESHOLD) {
        return false;
    }

    if peer_manager.connected_peers() == 0 {
        return false;
    }

    match tangle.get_milestone(tangle.get_latest_milestone_index()).await {
        Some(milestone) => {
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_secs() as u64)
                .saturating_sub(milestone.timestamp())
                <= HEALTH_MILESTONE_AGE_MAX
        }
        None => false,
    }
}
