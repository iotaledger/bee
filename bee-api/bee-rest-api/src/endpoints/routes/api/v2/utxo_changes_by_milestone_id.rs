// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_message::payload::milestone::MilestoneId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::endpoints::{
    config::ROUTE_UTXO_CHANGES_BY_MILESTONE_ID,
    filters::{with_storage, with_tangle},
    path_params::milestone_id,
    permission::has_permission,
    rejection::CustomRejection,
    routes::api::v2::utxo_changes_by_milestone_index,
    storage::StorageBackend,
};

fn path() -> impl Filter<Extract = (MilestoneId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("milestones"))
        .and(milestone_id())
        .and(warp::path("utxo-changes"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(
            ROUTE_UTXO_CHANGES_BY_MILESTONE_ID,
            public_routes,
            allowed_ips,
        ))
        .and(with_tangle(tangle))
        .and(with_storage(storage))
        .and_then(
            |milestone_id, tangle, storage| async move { utxo_changes_by_milestone_id(milestone_id, tangle, storage) },
        )
        .boxed()
}

pub(crate) fn utxo_changes_by_milestone_id<B: StorageBackend>(
    milestone_id: MilestoneId,
    tangle: ResourceHandle<Tangle<B>>,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let milestone_index = match tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => milestone_payload.essence().index(),
        None => return Err(reject::custom(CustomRejection::NotFound("data not found".to_string()))),
    };

    utxo_changes_by_milestone_index::utxo_changes_by_milestone_index(milestone_index, storage)
}
