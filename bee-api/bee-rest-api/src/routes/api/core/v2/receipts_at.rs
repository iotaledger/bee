// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_block::payload::milestone::MilestoneIndex;
use bee_ledger::types::Receipt;
use bee_storage::access::Fetch;
use log::error;

use crate::{
    error::ApiError,
    extractors::path::CustomPath,
    storage::StorageBackend,
    types::{dtos::ReceiptDto, responses::ReceiptsResponse},
    ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/receipts/:milestone_index", get(receipts_at::<B>))
}

async fn receipts_at<B: StorageBackend>(
    CustomPath(milestone_index): CustomPath<MilestoneIndex>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<ReceiptsResponse, ApiError> {
    let mut receipts_dto = Vec::new();

    if let Some(receipts) =
        Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&*args.storage, &milestone_index).map_err(|e| {
            error!("cannot fetch from storage: {}", e);
            ApiError::InternalServerError
        })?
    {
        for receipt in receipts {
            receipts_dto.push(ReceiptDto::from(receipt));
        }
    }

    Ok(ReceiptsResponse { receipts: receipts_dto })
}
