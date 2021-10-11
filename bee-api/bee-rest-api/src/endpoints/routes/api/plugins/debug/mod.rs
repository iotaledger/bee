// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod white_flag;

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend};

use bee_protocol::workers::{MessageRequesterWorker, RequestedMessages};
use bee_runtime::{event::Bus, resource::ResourceHandle};
use bee_tangle::Tangle;

use warp::{self, Filter, Rejection, Reply};

use std::net::IpAddr;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("debug"))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<Tangle<B>>,
    bus: ResourceHandle<Bus<'static>>,
    message_requester: MessageRequesterWorker,
    requested_messages: ResourceHandle<RequestedMessages>,
    rest_api_config: RestApiConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    white_flag::filter(
        public_routes,
        allowed_ips,
        storage,
        tangle,
        bus,
        message_requester,
        requested_messages,
        rest_api_config,
    )
}
