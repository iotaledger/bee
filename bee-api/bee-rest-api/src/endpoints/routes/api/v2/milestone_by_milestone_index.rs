// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::payload::milestone::MilestoneIndex;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::responses::MilestoneResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route(
        "/milestones/by-index/:milestone_index",
        get(milestone_by_milestone_index::<B>),
    )
}

pub(crate) async fn milestone_by_milestone_index<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
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
