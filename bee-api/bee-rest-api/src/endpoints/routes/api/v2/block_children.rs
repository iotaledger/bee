// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_block::BlockId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_BLOCK_CHILDREN, filters::with_tangle, path_params::block_id, permission::has_permission,
        storage::StorageBackend,
    },
    types::responses::BlockChildrenResponse,
};

fn path() -> impl Filter<Extract = (BlockId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("blocks"))
        .and(block_id())
        .and(warp::path("children"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_BLOCK_CHILDREN, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(|block_id, tangle| async move { block_children(block_id, tangle) })
        .boxed()
}

pub fn block_children<B: StorageBackend>(
    block_id: BlockId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    let mut children = Vec::from_iter(tangle.get_children(&block_id).unwrap_or_default());
    let count = children.len();
    let max_results = 1000;
    children.truncate(max_results);
    Ok(warp::reply::json(&BlockChildrenResponse {
        block_id: block_id.to_string(),
        max_results,
        count,
        children_block_ids: children.iter().map(BlockId::to_string).collect(),
    }))
}
