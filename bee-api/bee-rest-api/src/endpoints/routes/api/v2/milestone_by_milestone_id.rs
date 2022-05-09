// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::payload::milestone::MilestoneId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE_BY_MILESTONE_ID, filters::with_tangle, path_params::milestone_id,
        permission::has_permission, rejection::CustomRejection, storage::StorageBackend,
    },
    types::responses::MilestoneResponse,
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
        .and(has_permission(
            ROUTE_MILESTONE_BY_MILESTONE_ID,
            public_routes,
            allowed_ips,
        ))
        .and(with_tangle(tangle))
        .and_then(|milestone_id, tangle| async move { milestone_by_milestone_id(milestone_id, tangle) })
        .boxed()
}

pub(crate) fn milestone_by_milestone_id<B: StorageBackend>(
    milestone_id: MilestoneId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => Ok(warp::reply::json(&MilestoneResponse((&milestone_payload).into()))),
        None => Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
    }
}
