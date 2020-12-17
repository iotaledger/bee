// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::{NotFound, ServiceUnavailable},
    handlers::{EnvelopeContent, SuccessEnvelope},
    storage::Backend,
};

use bee_common_pt2::node::ResHandle;
use bee_ledger::model::Spent;
use bee_message::prelude::*;
use bee_storage::access::Fetch;

use serde::Serialize;
use warp::{reject, Rejection, Reply};

use std::ops::Deref;

pub(crate) async fn balance_ed25519<B: Backend>(
    addr: Ed25519Address,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    match Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage.deref(), &addr)
        .await
        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
    {
        Some(mut ids) => {
            let max_results = 1000;
            let count = ids.len();
            ids.truncate(max_results);
            let mut balance = 0;
            for id in ids {
                if let Some(output) = Fetch::<OutputId, bee_ledger::model::Output>::fetch(storage.deref(), &id)
                    .await
                    .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
                {
                    if let None = Fetch::<OutputId, Spent>::fetch(storage.deref(), &id)
                        .await
                        .map_err(|_| reject::custom(ServiceUnavailable("can not fetch from storage".to_string())))?
                    {
                        match output.inner() {
                            Output::SignatureLockedSingle(o) => balance += o.amount().get() as u64,
                            _ => {
                                return Err(reject::custom(ServiceUnavailable(
                                    "output type not supported".to_string(),
                                )))
                            }
                        }
                    }
                }
            }
            Ok(warp::reply::json(&SuccessEnvelope::new(GetBalanceForAddressResponse {
                address_type: 1,
                address: addr.to_string(),
                max_results,
                count,
                balance,
            })))
        }
        None => Err(reject::custom(NotFound("can not find output ids".to_string()))),
    }
}

/// Response of GET /api/v1/addresses/{address}
#[derive(Clone, Debug, Serialize)]
pub struct GetBalanceForAddressResponse {
    // The type of the address (0=WOTS, 1=Ed25519).
    #[serde(rename = "addressType")]
    pub address_type: u8,
    // hex encoded address
    pub address: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    pub balance: u64,
}

impl EnvelopeContent for GetBalanceForAddressResponse {}
