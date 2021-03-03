// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    rejection::CustomRejection,
    storage::StorageBackend,
    IS_SYNCED_THRESHOLD,
};

use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

pub(crate) async fn tips<B: StorageBackend>(tangle: ResourceHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    if !tangle.is_synced_threshold(IS_SYNCED_THRESHOLD) {
        return Err(reject::custom(CustomRejection::ServiceUnavailable(
            "the node is not synchronized".to_string(),
        )));
    }
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&SuccessBody::new(TipsResponse {
            tip_message_ids: tips.iter().map(|t| t.to_string()).collect(),
        }))),
        None => Err(reject::custom(CustomRejection::ServiceUnavailable(
            "tip pool is empty".to_string(),
        ))),
    }
}

/// Response of GET /api/v1/tips
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TipsResponse {
    #[serde(rename = "tipMessageIds")]
    pub tip_message_ids: Vec<String>,
}

impl BodyInner for TipsResponse {}
