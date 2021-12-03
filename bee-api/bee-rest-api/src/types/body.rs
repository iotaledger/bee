// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// A marker trait to represent the data that can be included into `SuccessBody` and `ErrorBody`.
pub trait BodyInner {}

/// Describes the response body of a successful HTTP request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuccessBody<T: BodyInner> {
    pub data: T,
}

impl<T: BodyInner> SuccessBody<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

/// Describes the response body of a unsuccessful HTTP request.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody<T: BodyInner> {
    pub error: T,
}

impl<T: BodyInner> ErrorBody<T> {
    pub fn new(error: T) -> Self {
        Self { error }
    }
}

/// Describes the default error format.
#[derive(Clone, Debug, Serialize)]
pub struct DefaultErrorResponse {
    pub code: String,
    pub message: String,
}

impl BodyInner for DefaultErrorResponse {}
