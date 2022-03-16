// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;
pub mod health;

use crate::endpoints::storage::StorageBackend;
use axum::Router;

pub(crate) fn filter_all<B: StorageBackend>() -> Router {
    Router::new().merge(api::filter::<B>()).merge(health::filter::<B>())
}
