// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{filters::with_args, rejection::CustomRejection, storage::StorageBackend, ApiArgsFullNode},
    types::{body::SuccessBody, dtos::ReceiptDto, responses::ReceiptsResponse},
};

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_storage::access::AsIterator;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("receipts")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|args| async move { receipts(args) })
        .boxed()
}

pub(crate) fn receipts<B: StorageBackend>(args: ApiArgsFullNode<B>) -> Result<impl Reply, Rejection> {
    let mut receipts_dto = Vec::new();
    let iterator = AsIterator::<(MilestoneIndex, Receipt), ()>::iter(&*args.storage)
        .map_err(|_| CustomRejection::InternalError)?;

    for result in iterator {
        let ((_, receipt), _) = result.map_err(|_| CustomRejection::InternalError)?;
        receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| CustomRejection::InternalError)?);
    }

    Ok(warp::reply::json(&SuccessBody::new(ReceiptsResponse {
        receipts: receipts_dto,
    })))
}
