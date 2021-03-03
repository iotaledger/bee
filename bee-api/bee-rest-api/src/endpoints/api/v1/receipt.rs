// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    rejection::CustomRejection,
    storage::StorageBackend,
    types::ReceiptDto,
};

use bee_ledger::model::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::{AsStream, Fetch};

use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use warp::{Rejection, Reply};

use std::convert::TryFrom;

/// Response of GET /api/v1/receipts/{milestone_index} and /api/v1/receipts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptsResponse(pub Vec<ReceiptDto>);

impl BodyInner for ReceiptsResponse {}

pub(crate) async fn receipts<B: StorageBackend>(storage: ResourceHandle<B>) -> Result<impl Reply, Rejection> {
    let mut receipts_dto = Vec::new();
    let mut stream = AsStream::<(MilestoneIndex, Receipt), ()>::stream(&*storage)
        .await
        .map_err(|_| CustomRejection::InternalError)?;

    while let Some(((_, receipt), _)) = stream.next().await {
        receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| CustomRejection::InternalError)?);
    }

    Ok(warp::reply::json(&SuccessBody::new(ReceiptsResponse(receipts_dto))))
}

pub(crate) async fn receipts_at<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let mut receipts_dto = Vec::new();

    if let Some(receipts) = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&*storage, &milestone_index)
        .await
        .map_err(|_| CustomRejection::InternalError)?
    {
        for receipt in receipts {
            receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| CustomRejection::InternalError)?);
        }
    }

    Ok(warp::reply::json(&SuccessBody::new(ReceiptsResponse(receipts_dto))))
}
