// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::ServiceUnavailable,
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
};

use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::Serialize;
use warp::{reject, Rejection, Reply};

pub(crate) async fn tips<B: StorageBackend>(tangle: ResourceHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&SuccessBody::new(TipsResponse {
            tip_1_message_id: tips.0.to_string(),
            tip_2_message_id: tips.1.to_string(),
        }))),
        None => Err(reject::custom(ServiceUnavailable("tip pool is empty".to_string()))),
    }
}

/// Response of GET /api/v1/tips
#[derive(Clone, Debug, Serialize)]
pub struct TipsResponse {
    #[serde(rename = "tip1MessageId")]
    pub tip_1_message_id: String,
    #[serde(rename = "tip2MessageId")]
    pub tip_2_message_id: String,
}

impl BodyInner for TipsResponse {}
