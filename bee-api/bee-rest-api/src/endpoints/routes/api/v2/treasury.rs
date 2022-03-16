// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{endpoints::storage::StorageBackend, types::responses::TreasuryResponse};

use bee_ledger::workers::storage;

use crate::endpoints::{error::ApiError, ApiArgsFullNode};
use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/treasury", get(treasury::<B>))
}

pub(crate) async fn treasury<B: StorageBackend>(
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let treasury = storage::fetch_unspent_treasury_output(&*args.storage).map_err(|_| ApiError::StorageBackend)?;

    Ok(Json(TreasuryResponse {
        milestone_id: treasury.milestone_id().to_string(),
        amount: treasury.inner().amount(),
    }))
}
