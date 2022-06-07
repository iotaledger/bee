// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, http::header::HeaderMap, response::IntoResponse, routing::get, Router};
use bee_block::{output::OutputId, payload::transaction::TransactionId, BlockId};
use bee_ledger::types::CreatedOutput;
use bee_storage::access::Fetch;
use log::error;

use crate::endpoints::{
    error::ApiError,
    extractors::path::CustomPath,
    routes::api::v2::blocks::{blocks_json, blocks_raw, BYTE_CONTENT_HEADER},
    storage::StorageBackend,
    ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/transactions/:transaction_id/included-block",
        get(transactions_included_block::<B>),
    )
}

async fn transactions_included_block<B: StorageBackend>(
    headers: HeaderMap,
    CustomPath(transaction_id): CustomPath<TransactionId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let block_id = get_block_id_from_transaction_id(args.clone(), transaction_id).await?;
    if let Some(value) = headers.get(axum::http::header::ACCEPT) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            return blocks_raw::<B>(block_id, args.clone()).await.map(|r| r.into_response());
        } else {
            blocks_json::<B>(block_id, args.clone())
                .await
                .map(|r| r.into_response())
        }
    } else {
        blocks_json::<B>(block_id, args.clone())
            .await
            .map(|r| r.into_response())
    }
}

async fn get_block_id_from_transaction_id<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
    transaction_id: TransactionId,
) -> Result<BlockId, ApiError> {
    // Safe to unwrap since 0 is a valid index;
    let output_id = OutputId::new(transaction_id, 0).unwrap();

    let fetched = Fetch::<OutputId, CreatedOutput>::fetch(&*args.storage, &output_id).map_err(|e| {
        error!("cannot fetch from storage: {}", e);
        ApiError::InternalError
    })?;

    match fetched {
        Some(output) => Ok(*output.block_id()),
        None => Err(ApiError::NotFound),
    }
}
