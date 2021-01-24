// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::StorageBackend;

use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use warp::{http::StatusCode, Reply};

use std::convert::Infallible;

pub(crate) async fn health<B: StorageBackend>(tangle: ResourceHandle<MsTangle<B>>) -> Result<impl Reply, Infallible> {
    if tangle.is_healthy().await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}
