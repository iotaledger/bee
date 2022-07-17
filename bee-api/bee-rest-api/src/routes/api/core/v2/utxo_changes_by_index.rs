// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_block::{output::OutputId, payload::milestone::MilestoneIndex};
use bee_ledger::types::OutputDiff;
use bee_storage::access::Fetch;
use log::error;

use crate::{
    error::ApiError, extractors::path::CustomPath, storage::StorageBackend, types::responses::UtxoChangesResponse,
    ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/milestones/by-index/:milestone_index/utxo-changes",
        get(utxo_changes_by_index::<B>),
    )
}

pub(crate) async fn utxo_changes_by_index<B: StorageBackend>(
    CustomPath(milestone_index): CustomPath<MilestoneIndex>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<UtxoChangesResponse, ApiError> {
    let fetched = Fetch::<MilestoneIndex, OutputDiff>::fetch(&*args.storage, &milestone_index)
        .map_err(|e| {
            error!("cannot fetch from storage: {}", e);
            ApiError::InternalServerError
        })?
        .ok_or(ApiError::NotFound)?;

    Ok(UtxoChangesResponse {
        index: *milestone_index,
        created_outputs: fetched.created_outputs().iter().map(OutputId::to_string).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(OutputId::to_string).collect(),
    })
}
