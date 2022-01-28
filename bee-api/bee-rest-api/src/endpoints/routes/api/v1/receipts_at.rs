// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_RECEIPTS_AT, filters::with_storage, path_params::milestone_index, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::{body::SuccessBody, dtos::ReceiptDto, responses::ReceiptsResponse},
};

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("receipts"))
        .and(milestone_index())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_RECEIPTS_AT, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(|milestone_index, storage| async move { receipts_at(milestone_index, storage) })
        .boxed()
}

pub(crate) fn receipts_at<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let mut receipts_dto = Vec::new();

    if let Some(receipts) = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&*storage, &milestone_index)
        .map_err(|_| CustomRejection::InternalError)?
    {
        for receipt in receipts {
            receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| CustomRejection::InternalError)?);
        }
    }

    Ok(warp::reply::json(&SuccessBody::new(ReceiptsResponse {
        receipts: receipts_dto,
    })))
}
