// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod essence;
mod transaction_id;

use crate::{unlock::UnlockBlocks, Error};

pub use essence::{Essence, RegularEssence, RegularEssenceBuilder};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};

use crypto::hashes::{blake2b::Blake2b256, Digest};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    essence: Essence,
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    pub const KIND: u32 = 0;

    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::default()
    }

    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_new());

        TransactionId::new(hasher.finalize().into())
    }

    pub fn essence(&self) -> &Essence {
        &self.essence
    }

    pub fn unlock_blocks(&self) -> &UnlockBlocks {
        &self.unlock_blocks
    }
}

impl Packable for TransactionPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.essence.packed_len() + self.unlock_blocks.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.essence.pack(writer)?;
        self.unlock_blocks.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let essence = Essence::unpack_inner::<R, CHECK>(reader)?;
        let unlock_blocks = UnlockBlocks::unpack_inner::<R, CHECK>(reader)?;

        Self::builder()
            .with_essence(essence)
            .with_unlock_blocks(unlock_blocks)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct TransactionPayloadBuilder {
    essence: Option<Essence>,
    unlock_blocks: Option<UnlockBlocks>,
}

impl TransactionPayloadBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_essence(mut self, essence: Essence) -> Self {
        self.essence.replace(essence);

        self
    }

    pub fn with_unlock_blocks(mut self, unlock_blocks: UnlockBlocks) -> Self {
        self.unlock_blocks.replace(unlock_blocks);

        self
    }

    pub fn finish(self) -> Result<TransactionPayload, Error> {
        let essence = self.essence.ok_or(Error::MissingField("essence"))?;
        let unlock_blocks = self.unlock_blocks.ok_or(Error::MissingField("unlock_blocks"))?;

        match essence {
            Essence::Regular(ref essence) => {
                if essence.inputs().len() != unlock_blocks.len() {
                    return Err(Error::InputUnlockBlockCountMismatch(
                        essence.inputs().len(),
                        unlock_blocks.len(),
                    ));
                }
            }
        }

        Ok(TransactionPayload { essence, unlock_blocks })
    }
}
