// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains a nonce provider that returns a given constant nonce.

use crate::providers::{NonceProvider, NonceProviderBuilder};

const DEFAULT_VALUE: u64 = 0;

/// Builder for the `Constant` nonce provider.
#[derive(Default)]
pub struct ConstantBuilder {
    value: Option<u64>,
}

impl ConstantBuilder {
    /// Sets the desired constant nonce for the `Constant` nonce provider.
    pub fn with_value(mut self, value: u64) -> Self {
        self.value = Some(value);
        self
    }
}

impl NonceProviderBuilder for ConstantBuilder {
    type Provider = Constant;

    fn new() -> Self {
        Self::default()
    }

    fn finish(self) -> Constant {
        Constant {
            value: self.value.unwrap_or(DEFAULT_VALUE),
        }
    }
}

/// A nonce provider that return constant nonces.
pub struct Constant {
    value: u64,
}

impl NonceProvider for Constant {
    type Builder = ConstantBuilder;
    type Error = std::convert::Infallible;

    fn nonce(&self, _: &[u8], _: f64) -> Result<u64, Self::Error> {
        Ok(self.value)
    }
}
