// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_block::{output::OutputId, payload::milestone::MilestoneIndex};
use bee_ledger::types::OutputDiff;
use bee_storage::access::Fetch;
use log::error;

use crate::{
    endpoints::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode},
    types::responses::UtxoChangesResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/milestones/by-index/:milestone_index/utxo-changes",
        get(utxo_changes_by_milestone_index::<B>),
    )
}

pub(crate) async fn utxo_changes_by_milestone_index<B: StorageBackend>(
    CustomPath(milestone_index): CustomPath<MilestoneIndex>,
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
