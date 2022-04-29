// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::payload::{
    milestone::{MilestoneId, MilestoneIndex},
    MilestonePayload,
};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;
use bee_tangle::milestone_metadata::MilestoneMetadata;
use log::error;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE, filters::with_storage, path_params::milestone_index, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::responses::MilestoneResponse,
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
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(|milestone_index, storage| async move { milestone_by_milestone_index(milestone_index, storage) })
        .boxed()
}

pub(crate) fn milestone_by_milestone_index<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let response = Fetch::<MilestoneIndex, MilestoneMetadata>::fetch(&*storage, &milestone_index).map_err(|e| {
        error!("cannot fetch from storage: {}", e);
        reject::custom(CustomRejection::InternalError)
    })?;

    match response {
        Some(milestone_metadata) => {
            let milestone_payload_response =
                Fetch::<MilestoneId, MilestonePayload>::fetch(&*storage, milestone_metadata.milestone_id()).map_err(
                    |e| {
                        error!("cannot fetch from storage: {}", e);
                        reject::custom(CustomRejection::InternalError)
                    },
                )?;

            match milestone_payload_response {
                Some(milestone_payload) => Ok(warp::reply::json(&MilestoneResponse((&milestone_payload).into()))),
                None => Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
            }
        }
        None => Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
    }
}
