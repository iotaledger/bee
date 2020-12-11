// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod balance_bech32;
pub mod balance_ed25519;
pub mod health;
pub mod info;
pub mod message;
pub mod message_children;
pub mod message_indexation;
pub mod message_metadata;
pub mod message_raw;
pub mod milestone;
pub mod output;
pub mod outputs_bech32;
pub mod outputs_ed25519;
pub mod submit_message;
pub mod submit_message_raw;
pub mod tips;

use serde::Serialize;

/// Marker trait
pub trait EnvelopeContent {}

#[derive(Clone, Debug, Serialize)]
pub struct SuccessEnvelope<T: EnvelopeContent> {
    pub data: T,
}

impl<T: EnvelopeContent> SuccessEnvelope<T> {
    pub(crate) fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorEnvelopeContent,
}

// Default error content
#[derive(Clone, Debug, Serialize)]
pub struct ErrorEnvelopeContent {
    pub code: String,
    pub message: String,
}

impl ErrorEnvelope {
    pub(crate) fn new(error: ErrorEnvelopeContent) -> Self {
        Self { error }
    }
}
