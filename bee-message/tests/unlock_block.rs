// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{error::UnlockBlockUnpackError, prelude::*};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn unpack_valid() {
    let mut bytes = vec![0, 0];
    bytes.extend(rand_bytes(32));
    bytes.extend(rand_bytes(64));

    let unlock_block = UnlockBlock::unpack_from_slice(bytes);

    assert!(unlock_block.is_ok());
}

#[test]
fn unpack_invalid_tag() {
    let mut bytes = vec![2, 0];
    bytes.extend(rand_bytes(32));
    bytes.extend(rand_bytes(64));

    let unlock_block = UnlockBlock::unpack_from_slice(bytes);

    assert!(matches!(
        unlock_block,
        Err(UnpackError::Packable(MessageUnpackError::UnlockBlock(
            UnlockBlockUnpackError::InvalidUnlockBlockKind(2)
        ))),
    ));
}
