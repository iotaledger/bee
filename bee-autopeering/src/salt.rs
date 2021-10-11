// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ring::rand::{SecureRandom as _, SystemRandom};

use std::time::{Duration, SystemTime};

const SALT_BYTE_SIZE: usize = 20;

pub(crate) struct Salt {
    rand_bytes: [u8; SALT_BYTE_SIZE],
    expiration: SystemTime,
}

impl Salt {
    pub fn new(lifetime: Duration) -> Self {
        let expiration = SystemTime::now()
            .checked_add(lifetime)
            .expect("system clock error or lifetime too long");

        let mut rand_bytes = [0u8; SALT_BYTE_SIZE];
        let mut crypto_rng = SystemRandom::new();
        crypto_rng
            .fill(&mut rand_bytes)
            .expect("error generating secure random bytes");

        Self { rand_bytes, expiration }
    }

    pub fn rand_bytes(&self) -> &[u8; SALT_BYTE_SIZE] {
        &self.rand_bytes
    }
}
