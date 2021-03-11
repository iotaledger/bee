// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_MILESTONE,
    filters::with_tangle,
    path_params::milestone_index,
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::{reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_index())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(milestone)
}

pub(crate) async fn milestone<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone_message_id(milestone_index).await {
        Some(message_id) => match tangle.get_metadata(&message_id).await {
            Some(metadata) => Ok(warp::reply::json(&SuccessBody::new(MilestoneResponse {
                milestone_index: *milestone_index,
                message_id: message_id.to_string(),
                timestamp: metadata.arrival_timestamp(),
            }))),
            None => Err(reject::custom(CustomRejection::NotFound(
                "can not find metadata for milestone".to_string(),
            ))),
        },
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find milestone".to_string(),
        ))),
    }
}

/// Response of GET /api/v1/milestone/{milestone_index}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneResponse {
    #[serde(rename = "index")]
    pub milestone_index: u32,
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub timestamp: u64,
}

impl BodyInner for MilestoneResponse {}
