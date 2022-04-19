// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable, PackableExt};

pub(crate) use self::essence::{InputCount, OutputCount};
pub use self::{
    essence::{RegularTransactionEssence, RegularTransactionEssenceBuilder, TransactionEssence},
    transaction_id::TransactionId,
};
use crate::{unlock_block::UnlockBlocks, Error};

/// A transaction to move funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    pub use super::essence::dto::TransactionEssenceDto;
    use super::*;
    use crate::{error::dto::DtoError, unlock_block::dto::UnlockBlockDto};

    /// The payload type to define a value transaction.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TransactionPayloadDto {
        #[serde(rename = "type")]
        pub kind: u32,
        pub essence: TransactionEssenceDto,
        #[serde(rename = "unlockBlocks")]
        pub unlock_blocks: Vec<UnlockBlockDto>,
    }

    impl From<&TransactionPayload> for TransactionPayloadDto {
        fn from(value: &TransactionPayload) -> Self {
            TransactionPayloadDto {
                kind: TransactionPayload::KIND,
                essence: value.essence().into(),
                unlock_blocks: value.unlock_blocks().iter().map(Into::into).collect::<Vec<_>>(),
            }
        }
    }

    impl TryFrom<&TransactionPayloadDto> for TransactionPayload {
        type Error = DtoError;

        fn try_from(value: &TransactionPayloadDto) -> Result<Self, Self::Error> {
            let mut unlock_blocks = Vec::new();
            for b in &value.unlock_blocks {
                unlock_blocks.push(b.try_into()?);
            }

            Ok(TransactionPayload::new(
                (&value.essence).try_into()?,
                UnlockBlocks::new(unlock_blocks)?,
            )?)
        }
    }
}
