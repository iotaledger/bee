// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
};

use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::{Rejection, Reply};

/// Response of GET /api/v1/treasury
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryResponse {}

impl BodyInner for TreasuryResponse {}

pub(crate) async fn treasury<B: StorageBackend>(_storage: ResourceHandle<B>) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&SuccessBody::new(TreasuryResponse {})))
}
