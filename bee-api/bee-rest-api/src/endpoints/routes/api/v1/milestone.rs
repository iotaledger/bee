// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE, filters::with_tangle, path_params::milestone_index, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::MilestoneResponse},
};

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_index())
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
        .and_then(|milestone_index, tangle| async move { milestone(milestone_index, tangle) })
        .boxed()
}

pub(crate) fn milestone<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone(milestone_index) {
        Some(milestone) => Ok(warp::reply::json(&SuccessBody::new(MilestoneResponse {
            milestone_index: *milestone_index,
            message_id: milestone.message_id().to_string(),
            timestamp: milestone.timestamp(),
        }))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "cannot find milestone".to_string(),
        ))),
    }
}
