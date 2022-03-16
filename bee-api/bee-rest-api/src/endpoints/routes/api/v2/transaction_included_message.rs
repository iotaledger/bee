// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{config::ROUTE_TRANSACTION_INCLUDED_MESSAGE, routes::api::v2::message, storage::StorageBackend};

use bee_ledger::types::CreatedOutput;
use bee_message::{output::OutputId, payload::transaction::TransactionId};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;
use bee_tangle::Tangle;

use std::net::IpAddr;

use crate::endpoints::{error::ApiError, ApiArgsFullNode};
use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/transactions/:transaction_id/included-message",
        get(transaction_included_message::<B>),
    )
}

pub(crate) async fn transaction_included_message<B: StorageBackend>(
    Path(transaction_id): Path<TransactionId>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    // Safe to unwrap since 0 is a valid index;
    let output_id = OutputId::new(transaction_id, 0).unwrap();

    match Fetch::<OutputId, CreatedOutput>::fetch(&*args.storage, &output_id)
        .map_err(|_| ApiError::ServiceUnavailable("Can not fetch from storage".to_string()))?
    {
        Some(output) => message::message(Path(*output.message_id()), Extension(args)).await,
        None => Err(ApiError::NotFound("Can not find output".to_string())),
    }
}
