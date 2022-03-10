// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;

pub use essence::{
    ChrysalisRegularTransactionEssence, ChrysalisRegularTransactionEssenceBuilder, ChrysalisTransactionEssence,
};

use crate::{unlock_block::UnlockBlocks, Error};

use crate::payload::transaction::TransactionId;
use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable, PackableExt};

/// A transaction to move funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ChrysalisTransactionPayload {
    essence: ChrysalisTransactionEssence,
    unlock_blocks: UnlockBlocks,
}

impl ChrysalisTransactionPayload {
    /// The payload kind of a [`TransactionPayload`].
    pub const KIND: u32 = 0;

    /// Creates a new [`TransactionPayload`].
    pub fn new(
        essence: ChrysalisTransactionEssence,
        unlock_blocks: UnlockBlocks,
    ) -> Result<ChrysalisTransactionPayload, Error> {
        verify_essence_unlock_blocks(&essence, &unlock_blocks)?;

        Ok(ChrysalisTransactionPayload { essence, unlock_blocks })
    }

    /// Return the essence of a [`TransactionPayload`].
    pub fn essence(&self) -> &ChrysalisTransactionEssence {
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

impl Packable for ChrysalisTransactionPayload {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.essence.pack(packer)?;
        self.unlock_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let essence = ChrysalisTransactionEssence::unpack::<_, VERIFY>(unpacker)?;
        let unlock_blocks = UnlockBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_essence_unlock_blocks(&essence, &unlock_blocks).map_err(UnpackError::Packable)?;
        }

        Ok(ChrysalisTransactionPayload { essence, unlock_blocks })
    }
}

fn verify_essence_unlock_blocks(
    essence: &ChrysalisTransactionEssence,
    unlock_blocks: &UnlockBlocks,
) -> Result<(), Error> {
    match essence {
        ChrysalisTransactionEssence::Regular(ref essence) => {
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
