// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_BALANCE_ED25519, filters::with_storage, path_params::ed25519_address, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::BalanceForAddressResponse},
};

use bee_ledger::types::Balance;
use bee_message::address::{Address, Ed25519Address};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use warp::{reject, Filter, Rejection, Reply};

use std::{net::IpAddr, ops::Deref};

fn path() -> impl Filter<Extract = (Ed25519Address,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(ed25519_address())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_BALANCE_ED25519, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(balance_ed25519)
}

pub(crate) async fn balance_ed25519<B: StorageBackend>(
    addr: Ed25519Address,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    match Fetch::<Address, Balance>::fetch(storage.deref(), &Address::Ed25519(addr))
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })? {
        Some(balance) => Ok(warp::reply::json(&SuccessBody::new(BalanceForAddressResponse {
            address_type: 1,
            address: addr.to_string(),
            balance: balance.amount(),
        }))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "balance not found".to_string(),
        ))),
    }
}
