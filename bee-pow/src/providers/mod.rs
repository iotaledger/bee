// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains nonce providers for Proof of Work.

mod constant;
mod miner;

pub use constant::{Constant, ConstantBuilder};
pub use miner::{Miner, MinerBuilder};

use std::sync::{atomic::AtomicBool, Arc};

/// A trait to build nonce providers.
pub trait NonceProviderBuilder: Default + Sized {
    /// The type of the built nonce provider.
    type Provider: NonceProvider<Builder = Self>;

    /// Creates a new nonce provider builder.
    fn new() -> Self;

    /// Constructs the nonce provider from the builder.
    fn finish(self) -> Self::Provider;
}

/// A trait describing how a nonce is provided.
pub trait NonceProvider: Sized {
    /// The type of the nonce provider builder.
    type Builder: NonceProviderBuilder<Provider = Self>;
    /// Type of errors occurring when providing nonces.
    type Error: std::error::Error;

    /// Returns a builder for this nonce provider.
    fn builder() -> Self::Builder {
        Self::Builder::default()
    }

    /// Provides a nonce given bytes and a target score.
    fn nonce(&self, bytes: &[u8], target_score: f64, done: Option<Arc<AtomicBool>>) -> Result<u64, Self::Error>;
}
