// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{prefix::StringPrefix, Packable};

use crate::{output::RentStructure, Error};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct ProtocolParemeters {
    /// The version of the protocol.
    version: u8,
    /// Then name of the network from which this snapshot was generated from.
    #[packable(unpack_error_with = |err| Error::InvalidNetworkName(err.into_item_err()))]
    network_name: StringPrefix<u8>,
    /// The human-readable part of the addresses within the network.
    #[packable(unpack_error_with = |err| Error::InvalidBech32Hrp(err.into_item_err()))]
    bech32_hrp: StringPrefix<u8>,
    /// The minimum PoW score.
    min_pow_score: u32,
    /// The below max depth parameter.
    below_max_depth: u8,
    /// The rent structure used by given node/network.
    rent_structure: RentStructure,
    /// The token supply.
    token_supply: u64,
}
