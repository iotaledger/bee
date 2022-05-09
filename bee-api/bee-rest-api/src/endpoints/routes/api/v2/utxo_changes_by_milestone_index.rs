// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::types::OutputDiff;
use bee_message::{milestone::MilestoneIndex, output::OutputId};
use bee_storage::access::Fetch;
use log::error;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::responses::UtxoChangesResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/milestones/by-index/:milestone_index/utxo-changes",
        get(milestone_utxo_changes::<B>),
    )
}

pub(crate) fn milestone_utxo_changes<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let fetched = Fetch::<MilestoneIndex, OutputDiff>::fetch(&*args.storage, &milestone_index)
        .map_err(|e| {
            error!("cannot fetch from storage: {}", e);
            ApiError::InternalError
        })?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(UtxoChangesResponse {
        index: *milestone_index,
        created_outputs: fetched.created_outputs().iter().map(OutputId::to_string).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(OutputId::to_string).collect(),
    }))
}
