// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_HEALTH,
    filters::{with_peer_manager, with_tangle},
    permission::has_permission,
    storage::StorageBackend,
};

use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use warp::{http::StatusCode, Filter, Rejection, Reply};

use std::{
    convert::Infallible,
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

const HEALTH_CONFIRMED_THRESHOLD: u32 = 2; // in milestones
const HEALTH_MILESTONE_AGE_MAX: u64 = 5 * 60; // in seconds

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::path("health").and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_HEALTH, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and(with_peer_manager(peer_manager))
        .and_then(health)
}

pub(crate) async fn health<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    peer_manager: ResourceHandle<PeerManager>,
) -> Result<impl Reply, Infallible> {
    if is_healthy(&tangle, &peer_manager).await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub async fn is_healthy<B: StorageBackend>(tangle: &MsTangle<B>, peer_manager: &PeerManager) -> bool {
    if !tangle.is_confirmed_threshold(HEALTH_CONFIRMED_THRESHOLD) {
        return false;
    }

    if peer_manager.connected_peers().await == 0 {
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
