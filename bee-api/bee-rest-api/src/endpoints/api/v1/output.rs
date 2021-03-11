// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_OUTPUT,
    filters::with_storage,
    path_params::output_id,
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
    types::OutputDto,
};

use bee_message::output::{ConsumedOutput, CreatedOutput, OutputId};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use serde::{Deserialize, Serialize};
use warp::{Filter, reject, Rejection, Reply};

use std::{convert::TryInto, net::IpAddr, ops::Deref};

fn path() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("outputs"))
        .and(output_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUT, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(output)
}

pub(crate) async fn output<B: StorageBackend>(
    output_id: OutputId,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let output = Fetch::<OutputId, CreatedOutput>::fetch(storage.deref(), &output_id)
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })?;
    let is_spent = Fetch::<OutputId, ConsumedOutput>::fetch(storage.deref(), &output_id)
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })?;
    match output {
        Some(output) => Ok(warp::reply::json(&SuccessBody::new(OutputResponse {
            message_id: output.message_id().to_string(),
            transaction_id: output_id.transaction_id().to_string(),
            output_index: output_id.index(),
            is_spent: is_spent.is_some(),
            output: output
                .inner()
                .try_into()
                .map_err(|e| reject::custom(CustomRejection::BadRequest(e)))?,
        }))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find output".to_string(),
        ))),
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
