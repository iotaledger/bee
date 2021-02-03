// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::ServiceUnavailable,
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
};

use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

pub(crate) async fn tips<B: StorageBackend>(tangle: ResourceHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&SuccessBody::new(TipsResponse {
            tip_message_ids: tips.iter().map(|t| t.to_string()).collect(),
        }))),
        None => Err(reject::custom(ServiceUnavailable("tip pool is empty".to_string()))),
    }
}

/// Response of GET /api/v1/tips
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TipsResponse {
    #[serde(rename = "tipMessageIds")]
    pub tip_message_ids: Vec<String>,
}

impl BodyInner for TipsResponse {}
