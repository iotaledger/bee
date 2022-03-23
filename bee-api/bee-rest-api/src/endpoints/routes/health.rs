// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{filters::with_args, storage::StorageBackend, ApiArgsFullNode};

use bee_protocol::workers::PeerManager;
use bee_tangle::Tangle;

use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};

const HEALTH_CONFIRMED_THRESHOLD: u32 = 2; // in milestones
const HEALTH_MILESTONE_AGE_MAX: u64 = 5 * 60; // in seconds

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::path("health").and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|args| async move { health(args) })
        .boxed()
}

pub(crate) fn health<B: StorageBackend>(args: ApiArgsFullNode<B>) -> Result<impl Reply, Infallible> {
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

    match tangle.get_milestone(tangle.get_latest_milestone_index()) {
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
