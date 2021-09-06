// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes, number::rand_number};

use bee_message::payload::salt_declaration::Salt;

/// Generates a random [`Salt`].
pub fn rand_salt() -> Salt {
    Salt::new(rand_bytes(96), rand_number()).unwrap()
}
