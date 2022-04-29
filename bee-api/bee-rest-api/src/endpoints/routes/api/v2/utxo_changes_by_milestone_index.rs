// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_ledger::types::OutputDiff;
use bee_message::{output::OutputId, payload::milestone::MilestoneIndex};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_UTXO_CHANGES_BY_MILESTONE_INDEX, filters::with_storage, path_params::milestone_index,
        permission::has_permission, rejection::CustomRejection, storage::StorageBackend,
    },
    types::responses::UtxoChangesResponse,
};

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(warp::path("by-index"))
        .and(milestone_index())
        .and(warp::path("utxo-changes"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(
            ROUTE_UTXO_CHANGES_BY_MILESTONE_INDEX,
            public_routes,
            allowed_ips,
        ))
        .and(with_storage(storage))
        .and_then(|index, storage| async move { utxo_changes_by_milestone_index(index, storage) })
        .boxed()
}

pub(crate) fn utxo_changes_by_milestone_index<B: StorageBackend>(
    index: MilestoneIndex,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let fetched = Fetch::<MilestoneIndex, OutputDiff>::fetch(&*storage, &index)
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })?
        .ok_or_else(|| {
            reject::custom(CustomRejection::NotFound(
                "can not find Utxo changes for given milestone index".to_string(),
            ))
        })?;

    Ok(warp::reply::json(&UtxoChangesResponse {
        index: *index,
        created_outputs: fetched.created_outputs().iter().map(OutputId::to_string).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(OutputId::to_string).collect(),
    }))
}
