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
    #[packable(unpack_error_with = |err| Error::InvalidNetworkName(err.into_item_err()))]
    network_name: StringPrefix<u8>,
    #[packable(unpack_error_with = |err| Error::InvalidBech32Hrp(err.into_item_err()))]
    bech32_hrp: StringPrefix<u8>,
    min_pow_score: u32,
    below_max_depth: u8,
    token_supply: u64,
}
