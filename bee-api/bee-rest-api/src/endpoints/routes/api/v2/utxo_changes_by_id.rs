// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_block::payload::milestone::MilestoneId;

use crate::{
    endpoints::{
        error::ApiError, extractors::path::CustomPath, routes::api::v2::utxo_changes_by_index, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::responses::UtxoChangesResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/milestones/:milestone_id/utxo-changes", get(utxo_changes_by_id::<B>))
}

async fn utxo_changes_by_id<B: StorageBackend>(
    CustomPath(milestone_id): CustomPath<MilestoneId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<UtxoChangesResponse, ApiError> {
    let milestone_index = match args.tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => milestone_payload.essence().index(),
        None => return Err(ApiError::NotFound),
    };

    utxo_changes_by_index::utxo_changes_by_index(CustomPath(milestone_index), Extension(args)).await
}
