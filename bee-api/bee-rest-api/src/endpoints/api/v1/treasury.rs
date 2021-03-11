// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_TREASURY,
    filters::with_storage,
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_ledger::storage;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("treasury"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_TREASURY, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(treasury)
}

pub(crate) async fn treasury<B: StorageBackend>(storage: ResourceHandle<B>) -> Result<impl Reply, Rejection> {
    let treasury = storage::fetch_unspent_treasury_output(&*storage)
        .await
        .map_err(|_| CustomRejection::StorageBackend)?;

    Ok(warp::reply::json(&SuccessBody::new(TreasuryResponse {
        milestone_id: treasury.milestone_id().to_string(),
        amount: treasury.inner().amount(),
    })))
}

/// Response of GET /api/v1/treasury
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryResponse {
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
    pub amount: u64,
}

impl BodyInner for TreasuryResponse {}
