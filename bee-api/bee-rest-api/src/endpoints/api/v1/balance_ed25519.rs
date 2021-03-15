// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_ledger::types::Balance;
use bee_message::address::{Address, Ed25519Address};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

use std::ops::Deref;

pub(crate) async fn balance_ed25519<B: StorageBackend>(
    addr: Ed25519Address,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    match Fetch::<Address, Balance>::fetch(storage.deref(), &Address::Ed25519(addr))
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })? {
        Some(balance) => Ok(warp::reply::json(&SuccessBody::new(BalanceForAddressResponse {
            address_type: 1,
            address: addr.to_string(),
            balance: balance.amount(),
        }))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "balance not found".to_string(),
        ))),
    }
}

/// Response of GET /api/v1/addresses/{address}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BalanceForAddressResponse {
    // The type of the address (1=Ed25519).
    #[serde(rename = "addressType")]
    pub address_type: u8,
    // hex encoded address
    pub address: String,
    pub balance: u64,
}

impl BodyInner for BalanceForAddressResponse {}
