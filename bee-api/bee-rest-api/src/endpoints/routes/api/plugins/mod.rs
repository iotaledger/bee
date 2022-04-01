// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod debug;

use axum::Router;

use crate::endpoints::storage::StorageBackend;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().nest("/plugins", debug::filter::<B>())
}
