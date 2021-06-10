// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_OUTPUTS_BECH32, filters::with_storage, path_params::bech32_address, permission::has_permission,
    routes::api::v1::outputs_ed25519::outputs_ed25519, storage::StorageBackend,
};

use bee_message::address::Address;
use bee_runtime::resource::ResourceHandle;

use warp::{Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (Address,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(bech32_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUTS_BECH32, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(|addr, storage| async move { outputs_bech32(addr, storage) })
}

pub(crate) fn outputs_bech32<B: StorageBackend>(
    addr: Address,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => outputs_ed25519(a, storage),
    }
}
