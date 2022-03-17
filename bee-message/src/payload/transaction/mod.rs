// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

pub(crate) use essence::{InputCount, OutputCount};
pub use essence::{RegularTransactionEssence, RegularTransactionEssenceBuilder, TransactionEssence};
pub use transaction_id::TransactionId;

use crate::{unlock_block::UnlockBlocks, Error};

use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable, PackableExt};

/// A transaction to move funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    essence: TransactionEssence,
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    /// The payload kind of a [`TransactionPayload`].
    pub const KIND: u32 = 6;

    /// Creates a new [`TransactionPayload`].
    pub fn new(essence: TransactionEssence, unlock_blocks: UnlockBlocks) -> Result<TransactionPayload, Error> {
        verify_essence_unlock_blocks(&essence, &unlock_blocks)?;

        Ok(TransactionPayload { essence, unlock_blocks })
    }

    /// Return the essence of a [`TransactionPayload`].
    pub fn essence(&self) -> &TransactionEssence {
        &self.essence
    }

    /// Return unlock blocks of a [`TransactionPayload`].
    pub fn unlock_blocks(&self) -> &UnlockBlocks {
        &self.unlock_blocks
    }

    /// Computes the identifier of a [`TransactionPayload`].
    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_to_vec());

        TransactionId::new(hasher.finalize().into())
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

        if VERIFY {
            verify_essence_unlock_blocks(&essence, &unlock_blocks).map_err(UnpackError::Packable)?;
        }

        Ok(TransactionPayload { essence, unlock_blocks })
    }
}

fn verify_essence_unlock_blocks(essence: &TransactionEssence, unlock_blocks: &UnlockBlocks) -> Result<(), Error> {
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

    Ok(())
}
