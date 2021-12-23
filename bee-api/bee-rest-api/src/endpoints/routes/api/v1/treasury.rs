// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_TREASURY, filters::with_storage, permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::TreasuryResponse},
};

use bee_ledger::workers::storage;
use bee_runtime::resource::ResourceHandle;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("treasury")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_TREASURY, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(|storage| async move { treasury(storage) })
        .boxed()
}

pub(crate) fn treasury<B: StorageBackend>(storage: ResourceHandle<B>) -> Result<impl Reply, Rejection> {
    let treasury = storage::fetch_unspent_treasury_output(&*storage).map_err(|_| CustomRejection::StorageBackend)?;

    Ok(warp::reply::json(&SuccessBody::new(TreasuryResponse {
        milestone_id: treasury.milestone_id().to_string(),
        amount: treasury.inner().amount(),
    })))
}
