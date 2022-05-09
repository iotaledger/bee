// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::types::CreatedOutput;
use bee_message::{output::OutputId, payload::transaction::TransactionId};
use bee_storage::access::Fetch;
use log::error;

use crate::endpoints::{error::ApiError, routes::api::v2::message, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/transactions/:transaction_id/included-message",
        get(transaction_included_message::<B>),
    )
}

pub(crate) fn transaction_included_message<B: StorageBackend>(
    Path(transaction_id): Path<TransactionId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    // Safe to unwrap since 0 is a valid index;
    let output_id = OutputId::new(transaction_id, 0).unwrap();

    let fetched = Fetch::<OutputId, CreatedOutput>::fetch(&*args.storage, &output_id).map_err(|e| {
        error!("cannot fetch from storage: {}", e);
        ApiError::InternalError
    })?;

    match fetched {
        Some(output) => message::message(Path(*output.message_id()), Extension(args)),
        None => Err(ApiError::NotFound),
    }
}
