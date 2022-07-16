// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, http::header::HeaderMap, routing::get, Router};
use bee_api_types::responses::MilestoneResponse;
use bee_block::payload::{milestone::MilestoneIndex, MilestonePayload};
use packable::PackableExt;

use crate::{
    error::ApiError, extractors::path::CustomPath, routes::api::core::v2::blocks::BYTE_CONTENT_HEADER,
    storage::StorageBackend, ApiArgsFullNode,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/milestones/by-index/:milestone_index", get(milestones_by_index::<B>))
}

async fn milestones_by_index<B: StorageBackend>(
    headers: HeaderMap,
    CustomPath(milestone_index): CustomPath<MilestoneIndex>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<MilestoneResponse, ApiError> {
    let milestone_payload = get_milestone_payload(args, milestone_index).await?;
    if let Some(value) = headers.get(axum::http::header::ACCEPT) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            return Ok(MilestoneResponse::Raw(milestone_payload.pack_to_vec()));
        }
    }
    Ok(MilestoneResponse::Json((&milestone_payload).into()))
}

async fn get_milestone_payload<B: StorageBackend>(
    args: ApiArgsFullNode<B>,
    milestone_index: MilestoneIndex,
) -> Result<MilestonePayload, ApiError> {
    let milestone_id = match args.tangle.get_milestone_metadata(milestone_index) {
        Some(milestone_metadata) => *milestone_metadata.milestone_id(),
        None => return Err(ApiError::NotFound),
    };
    match args.tangle.get_milestone(milestone_id) {
        Some(milestone_payload) => Ok(milestone_payload),
        None => Err(ApiError::NotFound),
    }
}
