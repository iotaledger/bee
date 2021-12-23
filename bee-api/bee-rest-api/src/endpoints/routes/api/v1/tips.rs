// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_TIPS, filters::with_tangle, permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend, CONFIRMED_THRESHOLD,
    },
    types::{body::SuccessBody, responses::TipsResponse},
};

use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("tips")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_TIPS, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(tips)
        .boxed()
}

pub(crate) async fn tips<B: StorageBackend>(tangle: ResourceHandle<Tangle<B>>) -> Result<impl Reply, Rejection> {
    if !tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
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
