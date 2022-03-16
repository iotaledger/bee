// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_RECEIPTS,  permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::{dtos::ReceiptDto, responses::ReceiptsResponse},
};

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::AsIterator;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::net::IpAddr;

use axum::extract::Extension;
use crate::endpoints::ApiArgsFullNode;
use axum::extract::Json;
use axum::Router;
use axum::routing::get;
use axum::response::IntoResponse;
use crate::endpoints::error::ApiError;
use std::sync::Arc;
use axum::extract::Path;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .route("/receipts", get(receipts::<B>))
}

pub(crate) async fn receipts<B: StorageBackend>(Extension(args): Extension<Arc<ApiArgsFullNode<B>>>) -> Result<impl IntoResponse, ApiError> {
    let mut receipts_dto = Vec::new();
    let iterator =
        AsIterator::<(MilestoneIndex, Receipt), ()>::iter(&*args.storage).map_err(|_| ApiError::InternalError)?;

    for result in iterator {
        let ((_, receipt), _) = result.map_err(|_| ApiError::InternalError)?;
        receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| ApiError::InternalError)?);
    }

    Ok(Json(ReceiptsResponse { receipts: receipts_dto }))
}
