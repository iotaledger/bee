// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::Error;

use crate::{
    proto,
    time::{self, Timestamp, HOUR},
};

use ring::rand::{SecureRandom as _, SystemRandom};

use std::time::{Duration, SystemTime};

const SALT_BYTE_LEN: usize = 20;
pub(crate) const SALT_LIFETIME_SECS: Duration = Duration::from_secs(2 * HOUR);

#[derive(Clone, Debug)]
pub struct Salt {
    pub(crate) bytes: [u8; SALT_BYTE_LEN],
    pub(crate) expiration_time: u64,
}

impl Salt {
    pub fn new(lifetime: Duration) -> Self {
        let expiration_time = time::unix_time_secs(
            SystemTime::now()
                .checked_add(lifetime)
                .expect("system clock error or lifetime too long"),
        );

        let mut rand_bytes = [0u8; SALT_BYTE_LEN];
        let crypto_rng = SystemRandom::new();
        crypto_rng
            .fill(&mut rand_bytes)
            .expect("error generating secure random bytes");

        Self {
            bytes: rand_bytes,
            expiration_time,
        }
    }

    pub fn bytes(&self) -> &[u8; SALT_BYTE_LEN] {
        &self.bytes
    }

    pub fn expiration_time(&self) -> u64 {
        self.expiration_time
    }
}

impl Default for Salt {
    fn default() -> Self {
        Self::new(SALT_LIFETIME_SECS)
    }
}

impl TryFrom<proto::Salt> for Salt {
    type Error = Error;

    fn try_from(salt: proto::Salt) -> Result<Self, Self::Error> {
        let proto::Salt { bytes, exp_time } = salt;

        Ok(Self {
            bytes: bytes.try_into().map_err(|_| Error::DeserializeFromProtobuf)?,
            expiration_time: exp_time,
        })
    }
}

pub(crate) fn is_expired(timestamp: Timestamp) -> bool {
    timestamp < time::unix_now_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Salt {
        pub(crate) fn new_zero_salt() -> Self {
            let expiration_time = time::unix_time_secs(
                SystemTime::now()
                    .checked_add(SALT_LIFETIME_SECS)
                    .expect("system clock error or lifetime too long"),
            );
            Self {
                bytes: [0u8; SALT_BYTE_LEN],
                expiration_time,
            }
        }
    }
}
