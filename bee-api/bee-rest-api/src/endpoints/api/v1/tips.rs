// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_TIPS,
    filters::with_tangle,
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
    IS_SYNCED_THRESHOLD,
};

use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::{Filter, reject, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("tips"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_TIPS, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(tips)
}

pub(crate) async fn tips<B: StorageBackend>(tangle: ResourceHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    if !tangle.is_synced_threshold(IS_SYNCED_THRESHOLD) {
        return Err(reject::custom(CustomRejection::ServiceUnavailable(
            "the node is not synchronized".to_string(),
        )));
    }
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&SuccessBody::new(TipsResponse {
            tip_message_ids: tips.iter().map(|t| t.to_string()).collect(),
        }))),
        None => Err(reject::custom(CustomRejection::ServiceUnavailable(
            "tip pool is empty".to_string(),
        ))),
    }
}

/// Response of GET /api/v1/tips
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TipsResponse {
    #[serde(rename = "tipMessageIds")]
    pub tip_message_ids: Vec<String>,
}

impl BodyInner for TipsResponse {}
