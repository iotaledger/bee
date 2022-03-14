// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0


use axum::{
    async_trait,
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};

use crate::types::body::DefaultErrorResponse;

#[derive(Debug, Clone)]
pub(crate) enum ApiError {
    Forbidden,
    BadRequest(String),
    NotFound(String),
    ServiceUnavailable(String),
    InternalError,
    StorageBackend,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "access forbidden"),
            ApiError::BadRequest(s) => (StatusCode::FORBIDDEN, &s),
            ApiError::NotFound(s) => (StatusCode::NOT_FOUND, &s),
            ApiError::ServiceUnavailable(s) => (StatusCode::SERVICE_UNAVAILABLE, &s),
            ApiError::InternalError => (StatusCode::INTERNAL_ERROR, "internal error"),
            ApiError::StorageBackend => (StatusCode::INTERNAL_ERROR, "internal error"),
        };

        let body = DefaultErrorResponse {
            code: status,
            message: error_message,
        };

        (status, 1).into_response()
    }
}