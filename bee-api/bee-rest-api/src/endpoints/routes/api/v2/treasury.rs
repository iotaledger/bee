// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_TREASURY, permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::responses::TreasuryResponse,
};

use bee_ledger::workers::storage;
use bee_runtime::resource::ResourceHandle;

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
        .route("/treasury", get(treasury::<B>))
}

pub(crate) async fn treasury<B: StorageBackend>(Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,) -> Result<impl IntoResponse, ApiError> {
    let treasury = storage::fetch_unspent_treasury_output(&*args.storage).map_err(|_| ApiError::StorageBackend)?;

    Ok(Json(TreasuryResponse {
        milestone_id: treasury.milestone_id().to_string(),
        amount: treasury.inner().amount(),
    }))
}
