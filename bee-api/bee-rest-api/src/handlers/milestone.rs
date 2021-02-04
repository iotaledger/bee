// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::NotFound,
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
};

use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

pub(crate) async fn milestone<B: StorageBackend>(
    milestone_index: MilestoneIndex,
    tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone_message_id(milestone_index).await {
        Some(message_id) => match tangle.get_metadata(&message_id).await {
            Some(metadata) => Ok(warp::reply::json(&SuccessBody::new(MilestoneResponse {
                milestone_index: *milestone_index,
                message_id: message_id.to_string(),
                timestamp: metadata.arrival_timestamp(),
            }))),
            None => Err(reject::custom(NotFound(
                "can not find metadata for milestone".to_string(),
            ))),
        },
        None => Err(reject::custom(NotFound("can not find milestone".to_string()))),
    }
}

/// Response of GET /api/v1/milestone/{milestone_index}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneResponse {
    #[serde(rename = "index")]
    pub milestone_index: u32,
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub timestamp: u64,
}

impl BodyInner for MilestoneResponse {}
