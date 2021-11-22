// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;

use axum::Router;

pub(crate) fn api_routes() -> Router {
    Router::new().nest("/api", api::api_routes())
}
