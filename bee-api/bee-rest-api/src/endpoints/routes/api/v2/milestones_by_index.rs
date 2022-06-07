// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_block::payload::milestone::MilestoneIndex;

use crate::{
    endpoints::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode},
    types::responses::MilestoneResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/milestones/by-index/:milestone_index", get(milestones_by_index::<B>))
}

pub(crate) async fn milestones_by_index<B: StorageBackend>(
    CustomPath(milestone_index): CustomPath<MilestoneIndex>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let milestone_id = match args.tangle.get_milestone_metadata(milestone_index) {
        Some(milestone_metadata) => *milestone_metadata.milestone_id(),
        None => return Err(ApiError::NotFound),
    };

    match args.tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => Ok(Json(MilestoneResponse((&milestone_payload).into()))),
        None => Err(ApiError::NotFound),
    }
}
