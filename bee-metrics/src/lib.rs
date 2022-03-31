// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Expose metrics via a prometheus-compatible client.

#![deny(missing_docs)]

pub mod encoding;
pub mod metrics;
mod registry;

use std::net::SocketAddr;

use axum::{
    extract::Extension,
    http::{header::HeaderName, HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Router, Server,
};
use prometheus_client::encoding::text::encode;

pub use self::registry::Registry;

async fn get_metrics(state: Extension<Registry>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/openmetrics-text; version=1.0.0; charset=utf-8"),
    );

    let mut encoded = Vec::new();
    // Panic: writing to a `Vec` cannot fail unless we run out of memory.`
    encode(&mut encoded, &state.0.registry.read()).unwrap();

    (StatusCode::OK, headers, encoded)
}

/// Serve the metrics registered in the provided [`Registry`] using the provided `SocketAddr`. This
/// endpoint can be scraped by Prometheus to collect the metrics.
pub async fn serve_metrics(addr: SocketAddr, registry: Registry) -> Result<(), axum::Error> {
    let app = Router::new()
        .route("/metrics", get(get_metrics))
        .layer(Extension(registry));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(axum::Error::new)?;

    Ok(())
}
