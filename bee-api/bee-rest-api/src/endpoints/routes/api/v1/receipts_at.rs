// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_storage::access::Fetch;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        filters::with_args, path_params::milestone_index, rejection::CustomRejection, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, dtos::ReceiptDto, responses::ReceiptsResponse},
};

fn path() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("receipts"))
        .and(milestone_index())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|milestone_index, args| async move { receipts_at(milestone_index, args) })
        .boxed()
}

pub(crate) fn receipts_at<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    let mut receipts_dto = Vec::new();

    if let Some(receipts) = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&*args.storage, &milestone_index)
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
