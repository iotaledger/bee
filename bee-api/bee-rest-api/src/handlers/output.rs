// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::{BadRequest, NotFound, ServiceUnavailable},
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
    types::OutputDto,
};

use bee_message::prelude::*;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

use std::{convert::TryInto, ops::Deref};

pub(crate) async fn output<B: StorageBackend>(
    output_id: OutputId,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let output = Fetch::<OutputId, CreatedOutput>::fetch(storage.deref(), &output_id)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?;
    let is_spent = Fetch::<OutputId, ConsumedOutput>::fetch(storage.deref(), &output_id)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?;
    match output {
        Some(output) => Ok(warp::reply::json(&SuccessBody::new(OutputResponse {
            message_id: output.message_id().to_string(),
            transaction_id: output_id.transaction_id().to_string(),
            output_index: output_id.index(),
            is_spent: is_spent.is_some(),
            output: output.inner().try_into().map_err(|e| reject::custom(BadRequest(e)))?,
        }))),
        None => Err(reject::custom(NotFound("can not find output".to_string()))),
    }
}

/// Response of GET /api/v1/outputs/{output_id}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "outputIndex")]
    pub output_index: u16,
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    pub output: OutputDto,
}

impl BodyInner for OutputResponse {}
