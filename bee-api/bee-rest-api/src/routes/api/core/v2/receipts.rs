// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_api_types::{dtos::ReceiptDto, responses::ReceiptsResponse};
use bee_block::payload::milestone::MilestoneIndex;
use bee_ledger::types::Receipt;
use bee_storage::access::AsIterator;
use log::error;

use crate::{error::ApiError, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/receipts", get(receipts::<B>))
}

async fn receipts<B: StorageBackend>(
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<ReceiptsResponse, ApiError> {
    let mut receipts_dto = Vec::new();
    let iterator = AsIterator::<(MilestoneIndex, Receipt), ()>::iter(&*args.storage).map_err(|e| {
        error!("cannot fetch from storage: {}", e);
        ApiError::InternalServerError
    })?;

    for result in iterator {
        let ((_, receipt), _) = result.map_err(|e| {
            error!("cannot iterate fetched receipts : {}", e);
            ApiError::InternalServerError
        })?;
        receipts_dto.push(ReceiptDto::from(receipt));
    }

    Ok(ReceiptsResponse { receipts: receipts_dto })
}
