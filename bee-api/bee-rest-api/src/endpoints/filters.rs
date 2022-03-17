// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{storage::StorageBackend, ApiArgsFullNode};

use warp::Filter;

use std::convert::Infallible;

pub(crate) fn with_args<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
) -> impl Filter<Extract = (ApiArgsFullNode<B>,), Error = Infallible> + Clone {
    warp::any().map(move || args.clone())
}
