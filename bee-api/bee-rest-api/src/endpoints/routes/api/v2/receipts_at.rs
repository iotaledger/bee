// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_RECEIPTS_AT, path_params::milestone_index, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::{dtos::ReceiptDto, responses::ReceiptsResponse},
};

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

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
        .route("/receipts/:milestone_index", get(receipts_at::<B>))
}

pub(crate) async fn receipts_at<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>
) -> Result<impl IntoResponse, ApiError> {
    let mut receipts_dto = Vec::new();

    if let Some(receipts) = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&*args.storage, &milestone_index)
        .map_err(|_| ApiError::InternalError)?
    {
        for receipt in receipts {
            receipts_dto.push(ReceiptDto::try_from(receipt).map_err(|_| ApiError::InternalError)?);
        }
    }

    Ok(Json(ReceiptsResponse { receipts: receipts_dto }))
}
