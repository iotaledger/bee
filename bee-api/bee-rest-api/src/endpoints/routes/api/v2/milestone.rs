// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::{
    milestone::MilestoneIndex,
    payload::{dto::MilestonePayloadDto, Payload},
};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE, filters::with_tangle, path_params::milestone_index, permission::has_permission,
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
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(milestone_by_milestone_index)
        .boxed()
}

pub(crate) async fn milestone_by_milestone_index<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone_message_id(milestone_index).await {
        Some(message_id) => match tangle.get(&message_id).await {
            Some(message) => {
                if let Some(Payload::Milestone(milestone_payload)) = message.payload() {
                    Ok(warp::reply::json(&MilestoneResponse(MilestonePayloadDto::from(
                        milestone_payload.as_ref(),
                    ))))
                } else {
                    unreachable!()
                }
            }
            None => Err(reject::custom(CustomRejection::NotFound(
                "can not find milestone".to_string(),
            ))),
        },
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find milestone".to_string(),
        ))),
    }
}
