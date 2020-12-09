// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub trait Provider {
    type Error: std::error::Error;

    fn nonce(&self, bytes: &[u8], target_score: f64) -> Result<u64, Self::Error>;
}
