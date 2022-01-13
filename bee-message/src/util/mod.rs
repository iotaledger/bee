// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod map;

#[cfg(not(feature = "std"))]
pub(crate) use map::{Map, Set};
#[cfg(feature = "std")]
pub(crate) use std::collections::{HashMap as Map, HashSet as Set};

use crate::error::Error;

use alloc::string::ToString;

/// Tries to decode an hexadecimal encoded string to an array of bytes.
pub fn hex_decode<const LENGTH: usize>(hex: &str) -> Result<[u8; LENGTH], Error> {
    <[u8; LENGTH]>::try_from(hex::decode(hex).map_err(|_| Error::InvalidHexadecimalChar(hex.to_string()))?).map_err(
        |_| Error::InvalidHexadecimalLength {
            expected: LENGTH * 2,
            actual: hex.len(),
        },
    )
}
