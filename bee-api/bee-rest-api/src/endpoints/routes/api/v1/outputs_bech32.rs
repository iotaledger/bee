// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    filters::with_args, path_params::bech32_address, routes::api::v1::outputs_ed25519::outputs_ed25519,
    storage::StorageBackend, ApiArgsFullNode,
};

use bee_message::address::Address;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

fn path() -> impl Filter<Extract = (Address,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(bech32_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(outputs_bech32(addr, args))
        .boxed()
}

pub(crate) async fn outputs_bech32<B: StorageBackend>(
    addr: Address,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => outputs_ed25519(a, args).await,
    }
}
