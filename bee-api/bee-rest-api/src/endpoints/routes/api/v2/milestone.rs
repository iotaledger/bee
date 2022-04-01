// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

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
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get_milestone_message_id(milestone_index).await {
        Some(message_id) => match args.tangle.get_metadata(&message_id).await {
            Some(metadata) => Ok(Json(MilestoneResponse {
                milestone_index: *milestone_index,
                message_id: message_id.to_string(),
                timestamp: metadata.arrival_timestamp(),
            })),
            None => Err(ApiError::NotFound("can not find metadata for milestone".to_string())),
        },
        None => Err(ApiError::NotFound("can not find milestone".to_string())),
    }
}
