// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, response::IntoResponse, routing::get, Router};
use bee_block::{output::OutputId, payload::transaction::TransactionId};
use bee_ledger::types::CreatedOutput;
use bee_storage::access::Fetch;
use log::error;

use crate::endpoints::{
    error::ApiError, extractors::path::CustomPath, routes::api::v2::blocks, storage::StorageBackend, ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/transactions/:transaction_id/included-block",
        get(transactions_included_block::<B>),
    )
}

async fn transactions_included_block<B: StorageBackend>(
    CustomPath(transaction_id): CustomPath<TransactionId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    // Safe to unwrap since 0 is a valid index;
    let output_id = OutputId::new(transaction_id, 0).unwrap();

    let fetched = Fetch::<OutputId, CreatedOutput>::fetch(&*args.storage, &output_id).map_err(|e| {
        error!("cannot fetch from storage: {}", e);
        ApiError::InternalError
    })?;

    match fetched {
        Some(output) => blocks::blocks_json(*output.block_id(), args).await,
        None => Err(ApiError::NotFound),
    }
}
