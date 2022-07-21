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

impl ProtocolParameters {
    /// Creates a new [`ProtocolParameters`].
    pub fn new(
        version: u8,
        network_name: String,
        bech32_hrp: String,
        min_pow_score: u32,
        below_max_depth: u8,
        rent_structure: RentStructure,
        token_supply: u64,
    ) -> Result<ProtocolParameters, Error> {
        Ok(ProtocolParameters {
            version,
            network_name: <StringPrefix<u8>>::try_from(network_name).map_err(Error::InvalidStringPrefix)?,
            bech32_hrp: <StringPrefix<u8>>::try_from(bech32_hrp).map_err(Error::InvalidStringPrefix)?,
            min_pow_score,
            below_max_depth,
            rent_structure,
            token_supply,
        })
    }

    /// Returns the version of the [`ProtocolParameters`].
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the network name of the [`ProtocolParameters`].
    pub fn network_name(&self) -> &str {
        &self.network_name
    }

    /// Returns the bech32 HRP of the [`ProtocolParameters`].
    pub fn bech32_hrp(&self) -> &str {
        &self.bech32_hrp
    }

    /// Returns the minimum PoW score of the [`ProtocolParameters`].
    pub fn min_pow_score(&self) -> u32 {
        self.min_pow_score
    }

    /// Returns the below max depth of the [`ProtocolParameters`].
    pub fn below_max_depth(&self) -> u8 {
        self.below_max_depth
    }

    /// Returns the rent structure of the [`ProtocolParameters`].
    pub fn rent_structure(&self) -> &RentStructure {
        &self.rent_structure
    }

    /// Returns the token supply of the [`ProtocolParameters`].
    pub fn token_supply(&self) -> u64 {
        self.token_supply
    }
}

#[cfg(feature = "inx")]
mod inx {
    use super::*;

    impl TryFrom<inx_bindings::proto::ProtocolParameters> for ProtocolParameters {
        type Error = crate::error::inx::InxError;

        fn try_from(value: inx_bindings::proto::ProtocolParameters) -> Result<Self, Self::Error> {
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
}
