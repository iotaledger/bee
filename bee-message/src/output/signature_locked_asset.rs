// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, payload::PAYLOAD_LENGTH_MAX, MessagePackError, MessageUnpackError, ValidationError};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
};

const ASSET_ID_LENGTH: usize = 32;

/// No `Vec` max length specified, so use `PAYLOAD_LENGTH_MAX` / length of `AddressBalance`.
const PREFIXED_BALANCES_LENGTH_MAX: usize = PAYLOAD_LENGTH_MAX / (ASSET_ID_LENGTH + core::mem::size_of::<u64>());

/// Error encountered packing a `SignatureLockedAssetOutput`.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SignatureLockedAssetPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible, u32>> for SignatureLockedAssetPackError {
    fn from(_: PackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefix
    }
}

impl fmt::Display for SignatureLockedAssetPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for asset balance vector"),
        }
    }
}

/// Error encountered unpacking a `SignatureLockedAssetOutput`.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SignatureLockedAssetUnpackError {
    InvalidAddressKind(u8),
    InvalidPrefix,
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    SignatureLockedAssetUnpackError,
    ValidationError,
    SignatureLockedAssetUnpackError::ValidationError
);

impl From<UnknownTagError<u8>> for SignatureLockedAssetUnpackError {
    fn from(error: UnknownTagError<u8>) -> Self {
        Self::InvalidAddressKind(error.0)
    }
}

impl From<UnpackPrefixError<Infallible, u32>> for SignatureLockedAssetUnpackError {
    fn from(_: UnpackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefix
    }
}

impl fmt::Display for SignatureLockedAssetUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddressKind(kind) => write!(f, "invalid address kind: {}", kind),
            Self::InvalidPrefix => write!(f, "invalid prefix for asset balance vector"),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// Tokenized asset balance information.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetBalance {
    /// The ID of the tokenized asset.
    id: [u8; ASSET_ID_LENGTH],
    /// The balance of the tokenized asset.
    balance: u64,
}

impl AssetBalance {
    /// Creates a new `AssetBalance`.
    pub fn new(id: [u8; 32], balance: u64) -> Self {
        Self { id, balance }
    }

    /// Returns the ID of an `AssetBalance`.
    pub fn id(&self) -> &[u8] {
        &self.id
    }

    /// Returns the balance of an `AssetBalance`.
    pub fn balance(&self) -> u64 {
        self.balance
    }
}

/// An output type which can be unlocked via a signature. It deposits onto one single address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedAssetOutput {
    address: Address,
    balances: Vec<AssetBalance>,
}

impl SignatureLockedAssetOutput {
    /// The output kind of a `SignatureLockedAssetOutput`.
    pub const KIND: u8 = 1;

    /// Creates a new `SignatureLockedAssetOutput`.
    pub fn new(address: Address, balances: Vec<AssetBalance>) -> Result<Self, ValidationError> {
        validate_balances_length(balances.len())?;

        Ok(Self { address, balances })
    }

    /// Returns the address of a `SignatureLockedAssetOutput`.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a `SignatureLockedAssetOutput`.
    pub fn balance_iter(&self) -> impl Iterator<Item = &AssetBalance> {
        self.balances.iter()
    }
}

impl Packable for SignatureLockedAssetOutput {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + 0u32.packed_len() + self.balances.len() * (ASSET_ID_LENGTH + 0u64.packed_len())
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.address.pack(packer).map_err(PackError::infallible)?;

        // Unwrap is safe, since length has been validated.
        let prefixed_balances: VecPrefix<AssetBalance, u32, PREFIXED_BALANCES_LENGTH_MAX> =
            self.balances.clone().try_into().unwrap();
        prefixed_balances
            .pack(packer)
            .map_err(PackError::coerce::<SignatureLockedAssetPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let address = Address::unpack(unpacker)
            .map_err(UnpackError::coerce::<SignatureLockedAssetUnpackError>)
            .map_err(UnpackError::coerce)?;

        let balances: Vec<AssetBalance> =
            VecPrefix::<AssetBalance, u32, PREFIXED_BALANCES_LENGTH_MAX>::unpack(unpacker)
                .map_err(UnpackError::coerce::<SignatureLockedAssetUnpackError>)
                .map_err(UnpackError::coerce)?
                .into();

        validate_balances_length(balances.len()).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { address, balances })
    }
}

fn validate_balances_length(len: usize) -> Result<(), ValidationError> {
    if len > PREFIXED_BALANCES_LENGTH_MAX {
        Err(ValidationError::InvalidAssetBalanceLength(len))
    } else {
        Ok(())
    }
}
