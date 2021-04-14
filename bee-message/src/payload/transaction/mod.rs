// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

use crate::{unlock::UnlockBlocks, Error};

pub use essence::{Essence, RegularEssence, RegularEssenceBuilder};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};

use crypto::hashes::{blake2b::Blake2b256, Digest};

/// A transaction message on the tangle.
///
/// A transaction consists of two parts:
/// 1. The transaction essence (inputs, outputs, optional payload)
/// 2. Unlock blocks, which unlock the input UTXOs consumed by the transaction
///
/// As of April 2021 (RFC draft stage) the possible input and output types are:
/// * [`UtxoInput`](crate::input::UtxoInput) - the output block from a previous transaction
/// * [`SignatureLockedSingleOutput`](crate::output::SignatureLockedSingleOutput) - deposit to a single address which is
///   unlocked via a signature
/// * [`SignatureLockedDustAllowanceOutput`](crate::output::SignatureLockedDustAllowanceOutput) - deposit to a single
///   address which also alters the dust allowance of the target address
///
/// And there are two types of unlock blocks:
/// * [`SignatureUnlock`](crate::unlock::SignatureUnlock)
/// * [`ReferenceUnlock`](crate::unlock::ReferenceUnlock)
///
/// Spec: #iota-protocol-rfc-draft
/// <https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md>
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    /// The inputs, outputs, and an optional embedded payload.
    essence: Essence,
    /// Unlocks the `essence`'s inputs.
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    /// The payload kind of a `TransactionPayload`.
    pub const KIND: u32 = 0;

    /// Return a new `TransactionPayloadBuilder` to build a `TransactionPayload`.
    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::default()
    }

    /// Computes the identifier of a `TransactionPayload`.
    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_new());

        TransactionId::new(hasher.finalize().into())
    }

    /// Return the essence of a `TransactionPayload`.
    pub fn essence(&self) -> &Essence {
        &self.essence
    }

    /// Return unlock blocks of a `TransactionPayload`.
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

/// A builder to build a `TransactionPayload`.
#[derive(Debug, Default)]
pub struct TransactionPayloadBuilder {
    essence: Option<Essence>,
    unlock_blocks: Option<UnlockBlocks>,
}

impl TransactionPayloadBuilder {
    /// Creates a new `TransactionPayloadBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an essence to a `TransactionPayloadBuilder`.
    pub fn with_essence(mut self, essence: Essence) -> Self {
        self.essence.replace(essence);

        self
    }

    /// Adds unlock blocks to a `TransactionPayloadBuilder`.
    pub fn with_unlock_blocks(mut self, unlock_blocks: UnlockBlocks) -> Self {
        self.unlock_blocks.replace(unlock_blocks);

        self
    }

    /// Finishes a `TransactionPayloadBuilder` into a `TransactionPayload`.
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
