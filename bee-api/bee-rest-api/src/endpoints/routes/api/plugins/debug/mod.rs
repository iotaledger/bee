// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod white_flag;

use crate::endpoints::{storage::StorageBackend, ApiArgsFullNode};

use warp::{self, Filter, Rejection, Reply};

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("debug"))
}

pub(crate) fn filter<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    white_flag::filter(args)
}
