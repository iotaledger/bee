// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_OUTPUTS_ED25519,
    filters::with_storage,
    path_params::ed25519_address,
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_message::{address::Ed25519Address, output::OutputId};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use serde::{Deserialize, Serialize};
use warp::{reject, Filter, Rejection, Reply};

use std::{net::IpAddr, ops::Deref};

fn path() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(ed25519_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUTS_ED25519, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(outputs_ed25519)
}

pub(crate) async fn outputs_ed25519<B: StorageBackend>(
    addr: Ed25519Address,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let mut fetched = match Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage.deref(), &addr)
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })? {
        Some(ids) => ids,
        None => vec![],
    };

    let count = fetched.len();
    let max_results = 1000;
    fetched.truncate(max_results);

    Ok(warp::reply::json(&SuccessBody::new(OutputsForAddressResponse {
        address_type: 1,
        address: addr.to_string(),
        max_results,
        count,
        output_ids: fetched.iter().map(|id| id.to_string()).collect(),
    })))
}

/// Response of GET /api/v1/addresses/{address}/outputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputsForAddressResponse {
    // The type of the address (1=Ed25519).
    #[serde(rename = "addressType")]
    pub address_type: u8,
    pub address: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "outputIds")]
    pub output_ids: Vec<String>,
}

impl BodyInner for OutputsForAddressResponse {}
