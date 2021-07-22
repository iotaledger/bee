// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

use crate::{
    unlock::{UnlockBlocks, UnlockBlocksPackError, UnlockBlocksUnpackError},
    MessagePackError, MessageUnpackError, ValidationError,
};

pub use essence::{
    TransactionEssence, TransactionEssenceBuilder, TransactionEssencePackError, TransactionEssenceUnpackError,
};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};

use bee_packable::{error::PackPrefixError, PackError, Packable, Packer, UnpackError, Unpacker};
use crypto::hashes::{blake2b::Blake2b256, Digest};

use alloc::boxed::Box;
use core::{convert::Infallible, fmt};

/// Error encountered packing a transaction payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum TransactionPackError {
    InvalidUnlockBlocksPrefix,
    TransactionEssence(Box<TransactionEssencePackError>),
    UnlockBlocks(UnlockBlocksPackError),
}

impl_wrapped_variant!(
    TransactionPackError,
    UnlockBlocksPackError,
    TransactionPackError::UnlockBlocks
);

impl From<TransactionEssencePackError> for TransactionPackError {
    fn from(error: TransactionEssencePackError) -> Self {
        Self::TransactionEssence(Box::new(error))
    }
}

impl From<PackPrefixError<Infallible, u16>> for TransactionPackError {
    fn from(error: PackPrefixError<Infallible, u16>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidUnlockBlocksPrefix,
        }
    }
}

impl fmt::Display for TransactionPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnlockBlocksPrefix => write!(f, "invalid unlock block vector prefix"),
            Self::UnlockBlocks(e) => write!(f, "error unpacking UnlockBlocks: {}", e),
            Self::TransactionEssence(e) => write!(f, "{}", e),
        }
    }
}

/// Error encountered unpacking a transaction payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum TransactionUnpackError {
    InvalidUnlockBlocksPrefix,
    TransactionEssence(Box<TransactionEssenceUnpackError>),
    UnlockBlocksUnpack(UnlockBlocksUnpackError),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    TransactionUnpackError,
    UnlockBlocksUnpackError,
    TransactionUnpackError::UnlockBlocksUnpack
);
impl_wrapped_variant!(
    TransactionUnpackError,
    ValidationError,
    TransactionUnpackError::ValidationError
);

impl From<TransactionEssenceUnpackError> for TransactionUnpackError {
    fn from(error: TransactionEssenceUnpackError) -> Self {
        match error {
            TransactionEssenceUnpackError::ValidationError(error) => Self::ValidationError(error),
            error => Self::TransactionEssence(Box::new(error)),
        }
    }
}

impl fmt::Display for TransactionUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnlockBlocksPrefix => write!(f, "invalid unlock block vector prefix"),
            Self::TransactionEssence(e) => write!(f, "error unpacking transaction essence: {}", e),
            Self::UnlockBlocksUnpack(e) => write!(f, "error unpacking unlock blocks: {}", e),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A transaction to move funds.
///
/// A `TransactionPayload` must:
/// * Ensure the number of `UnlockBlock`s matches the number of `Input`s.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    /// The version of the `TransactionPayload`.
    version: u8,
    /// The essence data making up a transaction.
    essence: TransactionEssence,
    /// Collection of unlock blocks relating to the transaction inputs.
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    /// The payload kind of a `TransactionPayload`.
    pub const KIND: u32 = 0;

    /// Return a new `TransactionPayloadBuilder` to build a `TransactionPayload`.
    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::new()
    }

    /// Computes the identifier of a `TransactionPayload`.
    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();
        hasher.update(Self::KIND.to_le_bytes());

        let bytes = self.pack_to_vec().unwrap();

        hasher.update(bytes);

        TransactionId::new(hasher.finalize().into())
    }

    /// Return the version of a `TransactionPayload`.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Return the essence of a `TransactionPayload`.
    pub fn essence(&self) -> &TransactionEssence {
        &self.essence
    }

    /// Return unlock blocks of a `TransactionPayload`.
    pub fn unlock_blocks(&self) -> &UnlockBlocks {
        &self.unlock_blocks
    }
}

impl Packable for TransactionPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len() + self.essence.packed_len() + self.unlock_blocks.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        self.essence.pack(packer)?;
        self.unlock_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let essence = TransactionEssence::unpack(unpacker)?;
        let unlock_blocks = UnlockBlocks::unpack(unpacker)?;

        validate_unlock_block_count(&essence, &unlock_blocks).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self {
            version,
            essence,
            unlock_blocks,
        })
    }
}

/// A builder to build a `TransactionPayload`.
#[derive(Debug, Default)]
pub struct TransactionPayloadBuilder {
    version: Option<u8>,
    essence: Option<TransactionEssence>,
    unlock_blocks: Option<UnlockBlocks>,
}

impl TransactionPayloadBuilder {
    /// Creates a new `TransactionPayloadBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a version to a `TransactionPayloadBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version.replace(version);
        self
    }

    /// Adds an essence to a `TransactionPayloadBuilder`.
    pub fn with_essence(mut self, essence: TransactionEssence) -> Self {
        self.essence.replace(essence);
        self
    }

    /// Adds unlock blocks to a `TransactionPayloadBuilder`.
    pub fn with_unlock_blocks(mut self, unlock_blocks: UnlockBlocks) -> Self {
        self.unlock_blocks.replace(unlock_blocks);
        self
    }

    /// Finishes a `TransactionPayloadBuilder` into a `TransactionPayload`.
    pub fn finish(self) -> Result<TransactionPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let essence = self.essence.ok_or(ValidationError::MissingField("essence"))?;
        let unlock_blocks = self
            .unlock_blocks
            .ok_or(ValidationError::MissingField("unlock_blocks"))?;

        validate_unlock_block_count(&essence, &unlock_blocks)?;

        Ok(TransactionPayload {
            version,
            essence,
            unlock_blocks,
        })
    }
}

fn validate_unlock_block_count(
    essence: &TransactionEssence,
    unlock_blocks: &UnlockBlocks,
) -> Result<(), ValidationError> {
    if essence.inputs().len() != unlock_blocks.len() {
        Err(ValidationError::InputUnlockBlockCountMismatch(
            essence.inputs().len(),
            unlock_blocks.len(),
        ))
    } else {
        Ok(())
    }
}
