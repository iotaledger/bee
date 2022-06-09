// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::types::body::{DefaultErrorResponse, ErrorBody};

// Errors that are returned to the user.
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("bad request: {0}")]
    DependencyError(DependencyError),
    #[error("not found")]
    NotFound,
    #[error("service unavailable: {0}")]
    ServiceUnavailable(&'static str),
    #[error("internal server error")]
    InternalServerError,
    #[error("forbidden")]
    Forbidden,
}

// Errors from dependencies that get exposed to the user.
#[derive(Error, Debug)]
pub enum DependencyError {
    #[error("{0}")]
    InvalidPath(#[from] axum::extract::rejection::PathRejection),
    #[error("{0}")]
    AxumJsonError(#[from] axum::extract::rejection::JsonRejection),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::error::Error),
    #[error("{0}")]
    InvalidBlockSubmitted(#[from] bee_protocol::workers::BlockSubmitterError),
    #[error("{0}")]
    InvalidBlock(#[from] bee_block::Error),
    #[error("{0}")]
    InvalidDto(#[from] bee_block::DtoError),
    #[error("{0}")]
    InvalidWhiteflag(#[from] bee_ledger::workers::error::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::DependencyError(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(ErrorBody::new(DefaultErrorResponse {
            code: status_code.as_u16().to_string(),
            message: self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
