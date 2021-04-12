// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE_UTXO_CHANGES, filters::with_storage, path_params::milestone_index,
        permission::has_permission, rejection::CustomRejection, storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::UtxoChangesResponse},
};

use bee_ledger::types::OutputDiff;
use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use warp::{reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_index())
        .and(warp::path("utxo-changes"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE_UTXO_CHANGES, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(milestone_utxo_changes)
}

pub(crate) async fn milestone_utxo_changes<B: StorageBackend>(
    index: MilestoneIndex,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let fetched = match Fetch::<MilestoneIndex, OutputDiff>::fetch(&*storage, &index)
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })? {
        Some(diff) => diff,
        None => {
            return Err(reject::custom(CustomRejection::NotFound(
                "can not find Utxo changes for given milestone index".to_string(),
            )));
        }
    };
    Ok(warp::reply::json(&SuccessBody::new(UtxoChangesResponse {
        index: *index,
        created_outputs: fetched.created_outputs().iter().map(|id| id.to_string()).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(|id| id.to_string()).collect(),
    })))
}
