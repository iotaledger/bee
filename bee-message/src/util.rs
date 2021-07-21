// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::error::ValidationError;

use alloc::borrow::ToOwned;
use core::convert::TryInto;

/// Tries to decode an hexadecimal encoded string to an array of bytes.
pub fn hex_decode<const N: usize>(hex: &str) -> Result<[u8; N], ValidationError> {
    hex::decode(hex)
        .map_err(|_| ValidationError::InvalidHexadecimalChar(hex.to_owned()))?
        .try_into()
        .map_err(|_| ValidationError::InvalidHexadecimalLength(N * 2, hex.len()))
}
