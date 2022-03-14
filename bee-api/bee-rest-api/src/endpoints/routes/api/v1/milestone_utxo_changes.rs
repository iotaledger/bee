// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, path_params::milestone_index, rejection::CustomRejection, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::UtxoChangesResponse},
};

use bee_ledger::types::OutputDiff;
use bee_message::milestone::MilestoneIndex;
use bee_storage::access::Fetch;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::sync::Arc;

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_index())
        .and(warp::path("utxo-changes"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgsFullNode<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|index, args| async move { milestone_utxo_changes(index, args) })
        .boxed()
}

pub(crate) fn milestone_utxo_changes<B: StorageBackend>(
    index: MilestoneIndex,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<impl Reply, Rejection> {
    let fetched = Fetch::<MilestoneIndex, OutputDiff>::fetch(&*args.storage, &index)
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

    Ok(warp::reply::json(&SuccessBody::new(UtxoChangesResponse {
        index: *index,
        created_outputs: fetched.created_outputs().iter().map(|id| id.to_string()).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(|id| id.to_string()).collect(),
    })))
}
