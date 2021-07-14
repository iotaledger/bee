// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{prelude::*, OutputUnpackError};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn unpack_valid() {
    let mut bytes = vec![0, 0];
    bytes.extend(rand_bytes(32));
    bytes.extend(vec![128, 0, 0, 0, 0, 0, 0, 0]);

    let output = Output::unpack_from_slice(bytes);

    assert!(output.is_ok());
}

#[test]
fn unpack_invalid_tag() {
    let mut bytes = vec![2, 0];
    bytes.extend(rand_bytes(32));
    bytes.extend(vec![128, 0, 0, 0, 0, 0, 0, 0]);

    let output = Output::unpack_from_slice(bytes);

    assert!(matches!(
        output,
        Err(UnpackError::Packable(MessageUnpackError::Output(OutputUnpackError::InvalidOutputKind(2)))),
    ));
}