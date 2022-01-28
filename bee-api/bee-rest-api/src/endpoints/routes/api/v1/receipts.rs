// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_RECEIPTS, filters::with_storage, permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, dtos::ReceiptDto, responses::ReceiptsResponse},
};

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::AsIterator;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("receipts")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_RECEIPTS, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(|storage| async move { receipts(storage) })
        .boxed()
}

pub(crate) fn receipts<B: StorageBackend>(storage: ResourceHandle<B>) -> Result<impl Reply, Rejection> {
    let mut receipts_dto = Vec::new();
    let iterator =
        AsIterator::<(MilestoneIndex, Receipt), ()>::iter(&*storage).map_err(|_| CustomRejection::InternalError)?;

    for result in iterator {
        let ((_, receipt), _) = result.map_err(|_| CustomRejection::InternalError)?;
        receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| CustomRejection::InternalError)?);
    }

    Ok(warp::reply::json(&SuccessBody::new(ReceiptsResponse {
        receipts: receipts_dto,
    })))
}
