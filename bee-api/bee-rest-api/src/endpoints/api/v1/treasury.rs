// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_ledger::storage;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::{Rejection, Reply};

/// Response of GET /api/v1/treasury
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryResponse {
    #[serde(rename = "milestoneId")]
    milestone_id: String,
    amount: u64,
}

impl BodyInner for TreasuryResponse {}

pub(crate) async fn treasury<B: StorageBackend>(storage: ResourceHandle<B>) -> Result<impl Reply, Rejection> {
    let treasury = storage::fetch_unspent_treasury_output(&*storage)
        .await
        .map_err(|_| CustomRejection::StorageBackend)?;

    Ok(warp::reply::json(&SuccessBody::new(TreasuryResponse {
        milestone_id: treasury.milestone_id().to_string(),
        amount: treasury.inner().amount(),
    })))
}
