// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::MessageUnpackError,
    input::{Input, InputUnpackError},
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn unpack_valid() {
    let mut bytes = vec![0];
    bytes.extend(rand_bytes(32));
    bytes.extend(vec![0, 0]);

    let input = Input::unpack_from_slice(bytes);

    assert!(input.is_ok());
}

#[test]
fn unpack_invalid_tag() {
    let mut bytes = vec![1];
    bytes.extend(rand_bytes(32));
    bytes.extend(vec![0, 0]);

    let input = Input::unpack_from_slice(bytes);

    assert!(matches!(
        input,
        Err(UnpackError::Packable(MessageUnpackError::Input(
            InputUnpackError::InvalidInputKind(1)
        ))),
    ));
}
