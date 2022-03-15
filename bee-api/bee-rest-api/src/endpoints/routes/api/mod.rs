// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod plugins;
pub mod v2;

use crate::endpoints::{storage::StorageBackend};

use axum::Router;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .nest("api",
              plugins::filter::<B>().merge(v2::filter::<B>())
        )
}
