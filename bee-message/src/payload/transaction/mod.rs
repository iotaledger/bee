// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod essence;
mod transaction_id;

use crate::{unlock::UnlockBlocks, Error};

pub use essence::{Essence, RegularEssence, RegularEssenceBuilder};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};

use crypto::hashes::{blake2b::Blake2b256, Digest};

pub(crate) const TRANSACTION_PAYLOAD_KIND: u32 = 0;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    essence: Essence,
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::default()
    }

    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();

        hasher.update(TRANSACTION_PAYLOAD_KIND.to_le_bytes());
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::builder()
            .with_essence(Essence::unpack(reader)?)
            .with_unlock_blocks(UnlockBlocks::unpack(reader)?)
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
        // TODO
        // inputs.sort();
        // outputs.sort();

        let essence = self.essence.ok_or(Error::MissingField("essence"))?;
        let unlock_blocks = self.unlock_blocks.ok_or(Error::MissingField("unlock_blocks"))?;

        match essence {
            Essence::Regular(ref essence) => {
                // Unlock Blocks validation
                if essence.inputs().len() != unlock_blocks.len() {
                    return Err(Error::InputUnlockBlockCountMismatch(
                        essence.inputs().len(),
                        unlock_blocks.len(),
                    ));
                }

                // for (i, block) in self.unlock_blocks.iter().enumerate() {
                //     // Signature Unlock Blocks must define an Ed25519-Signature
                //     match block {
                //         UnlockBlock::Reference(r) => {
                //             // Reference Unlock Blocks must specify a previous Unlock Block which is not of type
                // Reference             // Unlock Block. Since it's not the first input it unlocks, it
                // must have             // differente transaction id from previous one
                //             if i != 0 {
                //                 match &essence.inputs()[i] {
                //                     Input::UTXO(u) => match &essence.inputs()[i - 1] {
                //                         Input::UTXO(v) => {
                //                             if u.output_id().transaction_id() != v.output_id().transaction_id() {
                //                                 return Err(Error::InvalidIndex);
                //                             }
                //                         }
                //                     },
                //                 }
                //             }

                //             // The reference index must therefore be < the index of the Reference Unlock Block
                //             if r.index() >= i as u16 {
                //                 return Err(Error::InvalidIndex);
                //             }
                //         }
                //         UnlockBlock::Signature(_) => {
                //             // A Signature Unlock Block unlocking multiple inputs must only appear once (be unique)
                // and be             // positioned at same index of the first input it unlocks.
                //             if self.unlock_blocks.iter().filter(|j| *j == block).count() > 1 {
                //                 return Err(Error::DuplicateError);
                //             }

                //             // Since it's first input it unlocks, it must have differente transaction id from
                // previous one             if i != 0 {
                //                 match &essence.inputs()[i] {
                //                     Input::UTXO(u) => match &essence.inputs()[i - 1] {
                //                         Input::UTXO(v) => {
                //                             if u.output_id().transaction_id() == v.output_id().transaction_id() {
                //                                 return Err(Error::InvalidIndex);
                //                             }
                //                         }
                //                     },
                //                 }
                //             }
                //         }
                //     }
                // }
            }
        }

        Ok(TransactionPayload { essence, unlock_blocks })
    }
}
