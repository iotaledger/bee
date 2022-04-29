// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::payload::{milestone::MilestoneId, MilestonePayload};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;
use log::error;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE, filters::with_storage, path_params::milestone_id, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
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
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(|milestone_id, storage| async move { milestone_by_milestone_id(milestone_id, storage) })
        .boxed()
}

pub(crate) fn milestone_by_milestone_id<B: StorageBackend>(
    milestone_id: MilestoneId,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let response = Fetch::<MilestoneId, MilestonePayload>::fetch(&*storage, &milestone_id).map_err(|e| {
        error!("cannot fetch from storage: {}", e);
        reject::custom(CustomRejection::InternalError)
    })?;

    match response {
        Some(milestone_payload) => Ok(warp::reply::json(&MilestoneResponse((&milestone_payload).into()))),
        None => Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
    }
}
