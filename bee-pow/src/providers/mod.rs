// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod constant;
mod miner;

pub use constant::{Constant, ConstantBuilder};
pub use miner::{Miner, MinerBuilder};

use std::sync::{atomic::AtomicBool, Arc};

pub trait ProviderBuilder: Default + Sized {
    type Provider: Provider<Builder = Self>;

    fn new() -> Self;
    fn finish(self) -> Self::Provider;
}

pub trait Provider: Sized {
    type Builder: ProviderBuilder<Provider = Self>;
    type Error: std::error::Error;

    fn nonce(&self, bytes: &[u8], target_score: f64, done: Option<Arc<AtomicBool>>) -> Result<u64, Self::Error>;
}
