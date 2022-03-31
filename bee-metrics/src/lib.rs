// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Expose metrics via a prometheus-compatible client.

#![deny(missing_docs)]

pub mod encoding;
pub mod metrics;
mod registry;

use std::{net::SocketAddr, ops::Deref};

use axum::{
    extract::Extension,
    http::{header::HeaderName, HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Router, Server,
};
use prometheus_client::encoding::text::encode;

pub use self::registry::Registry;

async fn get_metrics<T>(state: Extension<T>) -> (StatusCode, HeaderMap, Vec<u8>)
where
    T: Deref<Target = Registry>,
{
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
pub async fn serve_metrics<T>(addr: SocketAddr, registry: T) -> Result<(), axum::Error>
where
    T: Deref<Target = Registry> + Clone + Send + Sync + 'static,
{
    let app = Router::new()
        .route("/metrics", get(get_metrics::<T>))
        .layer(Extension(registry));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(axum::Error::new)?;

    Ok(())
}
