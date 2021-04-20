// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_OUTPUT, filters::with_storage, path_params::output_id, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::OutputResponse},
};

use bee_ledger::types::{ConsumedOutput, CreatedOutput};
use bee_message::output::OutputId;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use warp::{reject, Filter, Rejection, Reply};

use std::net::IpAddr;

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
    let output = Fetch::<OutputId, CreatedOutput>::fetch(&*storage, &output_id)
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })?;
    let is_spent = Fetch::<OutputId, ConsumedOutput>::fetch(&*storage, &output_id)
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
            output: output.inner().into(),
        }))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find output".to_string(),
        ))),
    }
}
