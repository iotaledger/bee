// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::payload::milestone::MilestoneId;

use crate::endpoints::{
    error::ApiError, routes::api::v2::milestones_by_milestone_index, storage::StorageBackend, ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/milestones/:milestone_id", get(milestones_by_milestone_id::<B>))
}

pub(crate) async fn milestones_by_milestone_id<B: StorageBackend>(
    Path(milestone_id): Path<MilestoneId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let milestone_index = match args.tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => milestone_payload.essence().index(),
        None => return Err(ApiError::NotFound),
    };

    milestones_by_milestone_index::milestones_by_milestone_index(Path(milestone_index), Extension(args)).await
}
