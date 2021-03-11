// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ROUTE_BALANCE_BECH32,
    endpoints::api::v1::balance_ed25519::balance_ed25519, 
    filters::with_storage,
    path_params::bech32_address,
    permission::has_permission,
    rejection::CustomRejection, 
    storage::StorageBackend,
};

use bee_message::address::Address;
use bee_runtime::resource::ResourceHandle;

use warp::{Filter, reject, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (Address,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(bech32_address())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_BALANCE_BECH32, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(balance_bech32)
}

pub(crate) async fn balance_bech32<B: StorageBackend>(
    addr: Address,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => balance_ed25519(a, storage).await,
        _ => Err(reject::custom(CustomRejection::BadRequest(
            "address type not supported".to_string(),
        ))),
    }
}
