// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod debug;

use std::net::IpAddr;

use bee_protocol::workers::{BlockRequesterWorker, RequestedBlocks};
use bee_runtime::{event::Bus, resource::ResourceHandle};
use bee_tangle::Tangle;
use warp::{self, Filter, Rejection, Reply};

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend};

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("plugins"))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<Tangle<B>>,
    bus: ResourceHandle<Bus<'static>>,
    block_requester: BlockRequesterWorker,
    requested_blocks: ResourceHandle<RequestedBlocks>,
    rest_api_config: RestApiConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    debug::filter(
        public_routes,
        allowed_ips,
        storage,
        tangle,
        bus,
        block_requester,
        requested_blocks,
        rest_api_config,
    )
}
