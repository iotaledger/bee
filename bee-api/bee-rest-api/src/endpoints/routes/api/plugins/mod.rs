// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod debug;

use warp::{self, Filter, Rejection, Reply};

use crate::endpoints::{storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("plugins"))
}

pub(crate) fn filter<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    debug::filter(args)
}
