// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;
pub mod debug;
pub mod health;

use serde::Serialize;

/// Marker trait
pub trait BodyInner {}

#[derive(Clone, Debug, Serialize)]
pub struct SuccessBody<T: BodyInner> {
    pub data: T,
}

impl<T: BodyInner> SuccessBody<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody<T: BodyInner> {
    pub error: T,
}

impl<T: BodyInner> ErrorBody<T> {
    pub fn new(error: T) -> Self {
        Self { error }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DefaultErrorResponse {
    pub code: String,
    pub message: String,
}

impl BodyInner for DefaultErrorResponse {}
