// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod constants;
mod essence;
mod input;
mod output;
mod transaction_id;
mod treasury;
mod unlock;

use crate::Error;

pub use constants::{INPUT_OUTPUT_COUNT_MAX, INPUT_OUTPUT_COUNT_RANGE, INPUT_OUTPUT_INDEX_RANGE, IOTA_SUPPLY};
pub use essence::{Essence, RegularEssence, RegularEssenceBuilder};
pub use input::{Input, TreasuryInput, UTXOInput};
pub use output::{
    Address, Bech32Address, ConsumedOutput, CreatedOutput, Ed25519Address, Output, OutputId,
    SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput, TreasuryOutput, ED25519_ADDRESS_LENGTH,
    OUTPUT_ID_LENGTH,
};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};
pub use treasury::TreasuryTransactionPayload;
pub use unlock::{Ed25519Signature, ReferenceUnlock, SignatureUnlock, UnlockBlock};

use bee_common::packable::{Packable, Read, Write};

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};

use alloc::{boxed::Box, vec::Vec};
use core::{cmp::Ordering, slice::Iter};

pub(crate) const TRANSACTION_PAYLOAD_KIND: u32 = 0;
pub(crate) use treasury::TREASURY_TRANSACTION_PAYLOAD_KIND;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    essence: Essence,
    unlock_blocks: Box<[UnlockBlock]>,
}

impl TransactionPayload {
    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::default()
    }

    pub fn id(&self) -> TransactionId {
        let mut hasher = VarBlake2b::new(TRANSACTION_ID_LENGTH).unwrap();

        hasher.update(TRANSACTION_PAYLOAD_KIND.to_le_bytes());
        hasher.update(self.pack_new());

        let mut bytes = [0u8; TRANSACTION_ID_LENGTH];
        hasher.finalize_variable(|res| bytes.copy_from_slice(res));

        TransactionId::new(bytes)
    }

    pub fn essence(&self) -> &Essence {
        &self.essence
    }

    pub fn unlock_blocks(&self) -> &[UnlockBlock] {
        &self.unlock_blocks
    }

    pub fn unlock_block(&self, index: usize) -> &UnlockBlock {
        let unlock_block = &self.unlock_blocks[index];
        if let UnlockBlock::Reference(reference) = unlock_block {
            &self.unlock_blocks[reference.index() as usize]
        } else {
            unlock_block
        }
    }
}

impl Packable for TransactionPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.essence.packed_len()
            + 0u16.packed_len()
            + self.unlock_blocks.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.essence.pack(writer)?;

        (self.unlock_blocks.len() as u16).pack(writer)?;
        for unlock_block in self.unlock_blocks.as_ref() {
            unlock_block.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let essence = Essence::unpack(reader)?;

        let unlock_blocks_len = u16::unpack(reader)? as usize;

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&unlock_blocks_len) {
            return Err(Error::InvalidInputOutputCount(unlock_blocks_len));
        }

        let mut unlock_blocks = Vec::with_capacity(unlock_blocks_len);
        for _ in 0..unlock_blocks_len {
            unlock_blocks.push(UnlockBlock::unpack(reader)?);
        }

        Self::builder()
            .with_essence(essence)
            .with_unlock_blocks(unlock_blocks)
            .finish()
    }
}

#[allow(dead_code)]
fn is_sorted<T: Ord>(iterator: Iter<T>) -> bool {
    let mut iterator = iterator;
    let mut last = match iterator.next() {
        Some(e) => e,
        None => return true,
    };

    for curr in iterator {
        if let Ordering::Greater = &last.cmp(&curr) {
            return false;
        }
        last = curr;
    }

    true
}

#[derive(Debug, Default)]
pub struct TransactionPayloadBuilder {
    essence: Option<Essence>,
    unlock_blocks: Vec<UnlockBlock>,
}

impl TransactionPayloadBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_essence(mut self, essence: Essence) -> Self {
        self.essence = Some(essence);

        self
    }

    pub fn with_unlock_blocks(mut self, unlock_blocks: Vec<UnlockBlock>) -> Self {
        self.unlock_blocks = unlock_blocks;

        self
    }

    pub fn add_unlock_block(mut self, unlock_block: UnlockBlock) -> Self {
        self.unlock_blocks.push(unlock_block);

        self
    }

    pub fn finish(self) -> Result<TransactionPayload, Error> {
        // TODO
        // inputs.sort();
        // outputs.sort();

        let essence = self.essence.ok_or(Error::MissingField("essence"))?;

        match essence {
            Essence::Regular(ref essence) => {
                // Unlock Blocks validation
                if essence.inputs().len() != self.unlock_blocks.len() {
                    return Err(Error::InvalidUnlockBlockCount(
                        essence.inputs().len(),
                        self.unlock_blocks.len(),
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

        Ok(TransactionPayload {
            essence,
            unlock_blocks: self.unlock_blocks.into_boxed_slice(),
        })
    }
}
