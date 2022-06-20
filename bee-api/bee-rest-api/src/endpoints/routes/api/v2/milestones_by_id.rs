// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, http::header::HeaderMap, response::IntoResponse, routing::get, Json, Router};
use bee_block::payload::milestone::MilestoneId;
use packable::PackableExt;

use crate::{
    endpoints::{
        error::ApiError, extractors::path::CustomPath, routes::api::v2::blocks::BYTE_CONTENT_HEADER,
        storage::StorageBackend, ApiArgsFullNode,
    },
    types::responses::MilestoneResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/milestones/:milestone_id", get(milestones_by_id::<B>))
}

async fn milestones_by_id<B: StorageBackend>(
    headers: HeaderMap,
    CustomPath(milestone_id): CustomPath<MilestoneId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let milestone_payload = args
        .tangle
        .get_milestone(milestone_id)
        .ok_or_else(|| ApiError::NotFound)?;

    if let Some(value) = headers.get(axum::http::header::ACCEPT) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            return Ok(milestone_payload.pack_to_vec().into_response());
        }
    }

    Ok(Json(MilestoneResponse((&milestone_payload).into())).into_response())
}
