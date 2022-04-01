// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::types::body::{DefaultErrorResponse, ErrorBody};

pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    ServiceUnavailable(String),
    InternalError,
    StorageBackend,
    Forbidden,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "access forbidden".to_string()),
            ApiError::BadRequest(s) => (StatusCode::BAD_REQUEST, s),
            ApiError::NotFound(s) => (StatusCode::NOT_FOUND, s),
            ApiError::ServiceUnavailable(s) => (StatusCode::SERVICE_UNAVAILABLE, s),
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()),
            ApiError::StorageBackend => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()),
        };

        let body = Json(ErrorBody::new(DefaultErrorResponse {
            code: status.to_string(),
            message: error_message,
        }));

        (status, body).into_response()
    }
}
