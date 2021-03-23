// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod debug;

use crate::endpoints::storage::StorageBackend;

use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use warp::{self, Filter, Rejection, Reply};

use std::net::IpAddr;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("plugins"))
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    debug::filter(public_routes, allowed_ips, storage, tangle)
}
