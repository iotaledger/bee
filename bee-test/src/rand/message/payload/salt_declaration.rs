// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    number::{rand_number, rand_number_range},
};

use bee_message::payload::salt_declaration::{Salt, SaltDeclarationPayload};

/// Generates a random [`Salt`].
pub fn rand_salt() -> Salt {
    Salt::new(rand_bytes(rand_number_range(0..=255)), rand_number()).unwrap()
}

/// Generates a random [`SaltDeclarationPayload`].
pub fn rand_salt_declaration_payload() -> SaltDeclarationPayload {
    SaltDeclarationPayload::builder()
        .with_node_id(rand_number())
        .with_salt(rand_salt())
        .with_timestamp(rand_number())
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap()
}
