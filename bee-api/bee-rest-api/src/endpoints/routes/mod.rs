// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;
pub mod health;

use warp::{self, Filter, Rejection, Reply};

use crate::endpoints::{permission::check_permission, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter_all<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    check_permission(args.clone()).and(api::filter(args.clone()).or(health::filter(args)))
}
