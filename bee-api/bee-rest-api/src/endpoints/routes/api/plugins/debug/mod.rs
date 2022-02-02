// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod white_flag;

use crate::endpoints::{storage::StorageBackend, ApiArgs};

use warp::{self, Filter, Rejection, Reply};

use std::sync::Arc;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("debug"))
}

pub(crate) fn filter<B: StorageBackend>(
    args: Arc<ApiArgs<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    white_flag::filter(args)
}
