// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod plugins;
pub mod v1;

use crate::endpoints::{storage::StorageBackend, ApiArgs};

use warp::{self, Filter, Rejection, Reply};

use std::sync::Arc;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::path("api")
}

pub(crate) fn filter<B: StorageBackend>(
    args: Arc<ApiArgs<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    v1::filter(args.clone()).or(plugins::filter(args.clone()))
}
