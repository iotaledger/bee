// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains a nonce provider that returns a given constant nonce.

use crate::providers::{NonceProvider, NonceProviderBuilder};

impl NonceProviderBuilder for u64 {
    type Provider = u64;

    #[must_use]
    fn finish(self) -> u64 {
        self
    }
}

/// A nonce provider that returns constant nonces.
impl NonceProvider for u64 {
    type Builder = u64;
    type Error = std::convert::Infallible;

    fn nonce(&self, _: &[u8], _: u32) -> Result<u64, Self::Error> {
        Ok(*self)
    }
}
