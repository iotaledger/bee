// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{prefix::StringPrefix, Packable};

use crate::{output::RentStructure, Error};

/// Defines the parameters of the protocol.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct ProtocolParameters {
    // The version of the protocol running.
    version: u8,
    // The human friendly name of the network.
    #[packable(unpack_error_with = |err| Error::InvalidNetworkName(err.into_item_err()))]
    network_name: StringPrefix<u8>,
    // The HRP prefix used for Bech32 addresses in the network.
    #[packable(unpack_error_with = |err| Error::InvalidBech32Hrp(err.into_item_err()))]
    bech32_hrp: StringPrefix<u8>,
    // The minimum pow score of the network.
    min_pow_score: u32,
    // The below max depth parameter of the network.
    below_max_depth: u8,
    // The rent structure used by given node/network.
    rent_structure: RentStructure,
    // TokenSupply defines the current token supply on the network.
    token_supply: u64,
}

#[cfg(feature = "inx")]
impl TryFrom<inx::proto::ProtocolParameters> for ProtocolParameters {
    type Error = crate::error::inx::InxError;

    fn try_from(value: inx::proto::ProtocolParameters) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version as u8,
            network_name: <StringPrefix<u8>>::try_from(value.network_name)
                .map_err(|e| Self::Error::InvalidString(e.to_string()))?,
            bech32_hrp: <StringPrefix<u8>>::try_from(value.bech32_hrp)
                .map_err(|e| Self::Error::InvalidString(e.to_string()))?,
            min_pow_score: value.min_po_w_score,
            below_max_depth: value.below_max_depth as u8,
            rent_structure: value
                .rent_structure
                .ok_or(Self::Error::MissingField("rent_structure"))?
                .into(),
            token_supply: value.token_supply,
        })
    }
}
