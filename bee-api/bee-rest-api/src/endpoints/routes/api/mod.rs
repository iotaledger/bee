// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod plugins;
pub mod v1;

use warp::{self, Filter, Rejection, Reply};

use crate::endpoints::{storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::path("api")
}

pub(crate) fn filter<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    v1::filter(args.clone()).or(plugins::filter(args))
}
