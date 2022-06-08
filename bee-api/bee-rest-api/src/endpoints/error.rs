// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::types::body::{DefaultErrorResponse, ErrorBody};

#[derive(Error, Debug)]
pub enum ApiError {
    // Errors defined by the API.
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("not found")]
    NotFound,
    #[error("service unavailable: {0}")]
    ServiceUnavailable(&'static str),
    #[error("internal server error")]
    InternalServerError,
    #[error("forbidden")]
    Forbidden,
    // Errors from dependencies that can be displayed to the user.
    #[error("bad request: {0}")]
    InvalidPath(#[from] axum::extract::rejection::PathRejection),
    #[error("bad request: {0}")]
    AxumJsonError(#[from] axum::extract::rejection::JsonRejection),
    #[error("bad request: {0}")]
    SerdeJsonError(#[from] serde_json::error::Error),
    #[error("bad request: {0}")]
    InvalidBlockSubmitted(#[from] bee_protocol::workers::BlockSubmitterError),
    #[error("bad request: {0}")]
    InvalidBlock(#[from] bee_block::Error),
    #[error("bad request: {0}")]
    InvalidDto(#[from] bee_block::DtoError),
    #[error("bad request: {0}")]
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
            ApiError::InvalidPath(_) => StatusCode::BAD_REQUEST,
            ApiError::AxumJsonError(_) => StatusCode::BAD_REQUEST,
            ApiError::SerdeJsonError(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidBlockSubmitted(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidBlock(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidDto(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidWhiteflag(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(ErrorBody::new(DefaultErrorResponse {
            code: status_code.as_u16().to_string(),
            message: self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
