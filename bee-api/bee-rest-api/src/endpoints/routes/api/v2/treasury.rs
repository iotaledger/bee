// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::workers::storage;
use log::error;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::responses::TreasuryResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/treasury", get(treasury::<B>))
}

async fn treasury<B: StorageBackend>(
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let treasury = storage::fetch_unspent_treasury_output(&*args.storage).map_err(|e| {
        error!("cannot fetch from storage: {}", e);
        ApiError::InternalError
    })?;

    Ok(Json(TreasuryResponse {
        milestone_id: treasury.milestone_id().to_string(),
        amount: treasury.inner().amount().to_string(),
    }))
}
