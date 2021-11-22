// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod messages;

use axum::Router;

pub(crate) fn api_routes() -> Router {
    Router::new().nest("/messages", messages::api_routes())
}
