// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::providers::{Provider, ProviderBuilder};
use std::sync::{atomic::AtomicBool, Arc};

const DEFAULT_VALUE: u64 = 0;

#[derive(Default)]
pub struct ConstantBuilder {
    value: Option<u64>,
}

impl ConstantBuilder {
    pub fn with_value(mut self, value: u64) -> Self {
        self.value = Some(value);
        self
    }
}

impl ProviderBuilder for ConstantBuilder {
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

pub struct Constant {
    value: u64,
}

impl Provider for Constant {
    type Builder = ConstantBuilder;
    type Error = std::convert::Infallible;

    fn nonce(&self, _: &[u8], _: f64, _: Option<Arc<AtomicBool>>) -> Result<u64, Self::Error> {
        Ok(self.value)
    }
}
