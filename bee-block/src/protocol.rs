// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{prefix::StringPrefix, Packable};

use crate::Error;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct ProtocolParemeters {
    version: u8,
    network_name: StringPrefix<u8>,
    bech32_hrp: StringPrefix<u8>,
    min_pow_score: u32,
    below_max_depth: u8,
    token_supply: u64,
}
