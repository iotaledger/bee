// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod balance_bech32;
pub mod balance_ed25519;
pub mod health;
pub mod info;
pub mod message;
pub mod message_children;
pub mod message_metadata;
pub mod message_raw;
pub mod messages_find;
pub mod milestone;
pub mod output;
pub mod outputs_bech32;
pub mod outputs_ed25519;
pub mod peer;
pub mod peers;
pub mod submit_message;
pub mod submit_message_raw;
pub mod tips;

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
