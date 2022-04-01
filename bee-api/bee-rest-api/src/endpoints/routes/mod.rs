// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;
pub mod health;

use axum::Router;

use crate::endpoints::storage::StorageBackend;

pub(crate) fn filter_all<B: StorageBackend>() -> Router {
    Router::new().merge(api::filter::<B>()).merge(health::filter::<B>())
}
