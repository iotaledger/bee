// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::convert::Infallible;

use warp::Filter;

use crate::endpoints::{storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn with_args<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
) -> impl Filter<Extract = (ApiArgsFullNode<B>,), Error = Infallible> + Clone {
    warp::any().map(move || args.clone())
}
