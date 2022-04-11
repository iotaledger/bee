// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::types::body::{DefaultErrorResponse, ErrorBody};

pub enum ApiError {
    // Errors defined by the API
    BadRequest(&'static str),
    NotFound,
    ServiceUnavailable(&'static str),
    InternalError,
    Forbidden,
    // Errors from dependent crates which should be exposed to the user
    InvalidJsonProvided(serde_json::Error),
    InvalidMessageSubmitted(bee_protocol::workers::MessageSubmitterError),
    InvalidMessage(bee_message::Error),
    InvalidDto(bee_message::DtoError),
    InvalidWhiteflag(bee_ledger::workers::error::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::BadRequest(s) => (StatusCode::BAD_REQUEST, s.to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "could not find data".to_string()),
            ApiError::ServiceUnavailable(s) => (StatusCode::SERVICE_UNAVAILABLE, s.to_string()),
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "access forbidden".to_string()),
            ApiError::InvalidJsonProvided(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ApiError::InvalidMessageSubmitted(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ApiError::InvalidMessage(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ApiError::InvalidDto(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ApiError::InvalidWhiteflag(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        let body = Json(ErrorBody::new(DefaultErrorResponse {
            code: status.as_u16().to_string(),
            message: error_message,
        }));

        (status, body).into_response()
    }
}
