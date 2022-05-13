// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_block::BlockId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use packable::PackableExt;
use warp::{filters::BoxedFilter, http::Response, reject, Filter, Rejection, Reply};

use crate::endpoints::{
    config::ROUTE_MESSAGE_RAW, filters::with_tangle, path_params::message_id, permission::has_permission,
    rejection::CustomRejection, storage::StorageBackend,
};

fn path() -> impl Filter<Extract = (BlockId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path("raw"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_RAW, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(|message_id, tangle| async move { message_raw(message_id, tangle) })
        .boxed()
}

pub fn message_raw<B: StorageBackend>(
    message_id: BlockId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id) {
        Some(message) => Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(message.pack_to_vec())),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find message".to_string(),
        ))),
    }
}
