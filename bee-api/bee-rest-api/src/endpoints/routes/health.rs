// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    convert::Infallible,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{extract::Extension, http::StatusCode, response::IntoResponse, routing::get, Router};
use bee_protocol::workers::PeerManager;
use bee_tangle::Tangle;

use crate::endpoints::{storage::StorageBackend, ApiArgsFullNode};

const HEALTH_CONFIRMED_THRESHOLD: u32 = 2; // in milestones
const HEALTH_MILESTONE_AGE_MAX: Duration = Duration::from_secs(5 * 60);

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/health", get(health::<B>))
}

async fn health<B: StorageBackend>(
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, Infallible> {
    if is_healthy(&args.tangle, &args.peer_manager) {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub fn is_healthy<B: StorageBackend>(tangle: &Tangle<B>, peer_manager: &PeerManager) -> bool {
    if !tangle.is_confirmed_threshold(HEALTH_CONFIRMED_THRESHOLD) {
        return false;
    }

    if peer_manager.connected_peers() == 0 {
        return false;
    }

    match tangle.get_milestone_metadata(tangle.get_latest_milestone_index()) {
        Some(milestone) => {
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_secs() as u64)
                .saturating_sub(milestone.timestamp().into())
                <= HEALTH_MILESTONE_AGE_MAX.as_secs()
        }
        None => false,
    }
}
