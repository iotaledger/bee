// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

pub(crate) use essence::{InputCount, OutputCount};
pub use essence::{RegularTransactionEssence, TransactionEssence};
pub use transaction_id::TransactionId;

use crate::{unlock_block::UnlockBlocks, Error};

use bee_packable::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable, PackableExt};

use crypto::hashes::{blake2b::Blake2b256, Digest};

/// A builder to build a [`TransactionPayload`].
#[derive(Debug, Default)]
pub struct TransactionPayloadBuilder {
    essence: Option<TransactionEssence>,
    unlock_blocks: Option<UnlockBlocks>,
}

impl TransactionPayloadBuilder {
    /// Creates a new [`TransactionPayloadBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an essence to a [`TransactionPayloadBuilder`].
    pub fn with_essence(mut self, essence: TransactionEssence) -> Self {
        self.essence.replace(essence);
        self
    }

    /// Adds unlock blocks to a [`TransactionPayloadBuilder`].
    pub fn with_unlock_blocks(mut self, unlock_blocks: UnlockBlocks) -> Self {
        self.unlock_blocks.replace(unlock_blocks);
        self
    }

    /// Finishes a [`TransactionPayloadBuilder`] into a [`TransactionPayload`].
    pub fn finish(self) -> Result<TransactionPayload, Error> {
        let essence = self.essence.ok_or(Error::MissingField("essence"))?;
        let unlock_blocks = self.unlock_blocks.ok_or(Error::MissingField("unlock_blocks"))?;

        match essence {
            TransactionEssence::Regular(ref essence) => {
                if essence.inputs().len() != unlock_blocks.len() {
                    return Err(Error::InputUnlockBlockCountMismatch {
                        input_count: essence.inputs().len(),
                        block_count: unlock_blocks.len(),
                    });
                }
            }
        }

        Ok(TransactionPayload { essence, unlock_blocks })
    }
}

/// A transaction to move funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    essence: TransactionEssence,
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    /// The payload kind of a [`TransactionPayload`].
    pub const KIND: u32 = 0;

    /// Return a new [`TransactionPayloadBuilder`] to build a [`TransactionPayload`].
    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::default()
    }

    /// Computes the identifier of a [`TransactionPayload`].
    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_to_vec());

        TransactionId::new(hasher.finalize().into())
    }

    /// Return the essence of a [`TransactionPayload`].
    pub fn essence(&self) -> &TransactionEssence {
        &self.essence
    }

    /// Return unlock blocks of a [`TransactionPayload`].
    pub fn unlock_blocks(&self) -> &UnlockBlocks {
        &self.unlock_blocks
    }
}

impl Packable for TransactionPayload {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.essence.pack(packer)?;
        self.unlock_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let essence = TransactionEssence::unpack::<_, VERIFY>(unpacker)?;
        let unlock_blocks = UnlockBlocks::unpack::<_, VERIFY>(unpacker)?;

        Self::builder()
            .with_essence(essence)
            .with_unlock_blocks(unlock_blocks)
            .finish()
            .map_err(UnpackError::Packable)
    }
}
