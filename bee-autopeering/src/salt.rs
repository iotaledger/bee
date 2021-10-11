// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::time::unix;

use ring::rand::{SecureRandom as _, SystemRandom};

use std::time::{Duration, SystemTime};

const SALT_BYTE_SIZE: usize = 20;

pub(crate) struct Salt {
    bytes: [u8; SALT_BYTE_SIZE],
    expiration_time: u64,
}

impl Salt {
    pub fn new(lifetime: Duration) -> Self {
        let expiration_time = unix(
            SystemTime::now()
                .checked_add(lifetime)
                .expect("system clock error or lifetime too long"),
        );

        let mut rand_bytes = [0u8; SALT_BYTE_SIZE];
        let crypto_rng = SystemRandom::new();
        crypto_rng
            .fill(&mut rand_bytes)
            .expect("error generating secure random bytes");

        Self {
            bytes: rand_bytes,
            expiration_time,
        }
    }

    pub fn bytes(&self) -> &[u8; SALT_BYTE_SIZE] {
        &self.bytes
    }

    pub fn expiration_time(&self) -> u64 {
        self.expiration_time
    }
}
