// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod white_flag;

use crate::endpoints::storage::StorageBackend;
use axum::Router;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().nest("/debug", white_flag::filter::<B>())
}
