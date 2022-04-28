// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::payload::milestone::MilestoneId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::endpoints::{
    config::ROUTE_MILESTONE, filters::with_tangle, path_params::milestone_id, permission::has_permission,
    storage::StorageBackend,
};

fn path() -> impl Filter<Extract = (MilestoneId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(|milestone_id, tangle| async move { milestone_by_milestone_id(milestone_id, tangle) })
        .boxed()
}

pub(crate) fn milestone_by_milestone_id<B: StorageBackend>(
    milestone_id: MilestoneId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    Ok(warp::http::StatusCode::SERVICE_UNAVAILABLE)
}
