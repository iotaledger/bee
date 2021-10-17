// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::proto;
use crate::time::unix;

use prost::{bytes::BytesMut, EncodeError};
use prost::{DecodeError, Message};
use ring::rand::{SecureRandom as _, SystemRandom};

use std::{
    convert::TryInto,
    time::{Duration, SystemTime},
};

const SALT_BYTE_SIZE: usize = 20;

#[derive(Clone)]
pub(crate) struct Salt {
    pub(crate) bytes: [u8; SALT_BYTE_SIZE],
    pub(crate) expiration_time: u64,
}

impl Salt {
    pub(crate) fn new(lifetime: Duration) -> Self {
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

    pub(crate) fn bytes(&self) -> &[u8; SALT_BYTE_SIZE] {
        &self.bytes
    }

    pub(crate) fn expiration_time(&self) -> u64 {
        self.expiration_time
    }

    pub(crate) fn from_protobuf(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(proto::Salt::decode(bytes)?.into())
    }

    pub(crate) fn protobuf(&self) -> Result<BytesMut, EncodeError> {
        let salt = proto::Salt {
            bytes: self.bytes.to_vec(),
            exp_time: self.expiration_time,
        };

        let mut bytes = BytesMut::with_capacity(salt.encoded_len());
        salt.encode(&mut bytes)?;

        Ok(bytes)
    }
}

impl From<proto::Salt> for Salt {
    fn from(salt: proto::Salt) -> Self {
        let proto::Salt { bytes, exp_time } = salt;

        Self {
            bytes: bytes.try_into().expect("invalid salt length"),
            expiration_time: exp_time,
        }
    }
}
