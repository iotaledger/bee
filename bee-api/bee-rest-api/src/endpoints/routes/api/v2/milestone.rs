// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::milestone::MilestoneIndex;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::responses::MilestoneResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/milestones/:milestone_index", get(milestone::<B>))
}

pub(crate) async fn milestone<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get_milestone(milestone_index).await {
        Some(milestone) => Ok(Json(MilestoneResponse {
            milestone_index: *milestone_index,
            message_id: milestone.message_id().to_string(),
            timestamp: milestone.timestamp(),
        })),
        None => Err(ApiError::NotFound("cannot find milestone".to_string())),
    }
}
