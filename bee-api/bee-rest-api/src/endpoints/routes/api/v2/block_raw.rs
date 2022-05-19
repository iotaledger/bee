// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_block::BlockId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use packable::PackableExt;
use warp::{filters::BoxedFilter, http::Response, reject, Filter, Rejection, Reply};

use crate::endpoints::{
    config::ROUTE_BLOCK_RAW, filters::with_tangle, path_params::block_id, permission::has_permission,
    rejection::CustomRejection, storage::StorageBackend,
};

fn path() -> impl Filter<Extract = (BlockId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("blocks"))
        .and(block_id())
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
        .and(has_permission(ROUTE_BLOCK_RAW, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(|block_id, tangle| async move { block_raw(block_id, tangle) })
        .boxed()
}

pub fn block_raw<B: StorageBackend>(
    block_id: BlockId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&block_id) {
        Some(block) => Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(block.pack_to_vec())),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find block".to_string(),
        ))),
    }
}
