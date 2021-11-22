// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod v1;

use axum::Router;

pub(crate) fn api_routes() -> Router {
    Router::new().nest("/v1", v1::api_routes())
}
