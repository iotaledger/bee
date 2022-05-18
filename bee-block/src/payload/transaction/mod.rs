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
use crate::{unlock::Unlocks, Error};

/// A transaction to move funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    essence: TransactionEssence,
    unlocks: Unlocks,
}

impl TransactionPayload {
    /// The payload kind of a [`TransactionPayload`].
    pub const KIND: u32 = 6;

    /// Creates a new [`TransactionPayload`].
    pub fn new(essence: TransactionEssence, unlocks: Unlocks) -> Result<TransactionPayload, Error> {
        verify_essence_unlocks(&essence, &unlocks)?;

        Ok(TransactionPayload { essence, unlocks })
    }

    /// Return the essence of a [`TransactionPayload`].
    pub fn essence(&self) -> &TransactionEssence {
        &self.essence
    }

    /// Return unlock blocks of a [`TransactionPayload`].
    pub fn unlocks(&self) -> &Unlocks {
        &self.unlocks
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
        self.unlocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let essence = TransactionEssence::unpack::<_, VERIFY>(unpacker)?;
        let unlocks = Unlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_essence_unlocks(&essence, &unlocks).map_err(UnpackError::Packable)?;
        }

        Ok(TransactionPayload { essence, unlocks })
    }
}

fn verify_essence_unlocks(essence: &TransactionEssence, unlocks: &Unlocks) -> Result<(), Error> {
    match essence {
        TransactionEssence::Regular(ref essence) => {
            if essence.inputs().len() != unlocks.len() {
                return Err(Error::InputUnlockCountMismatch {
                    input_count: essence.inputs().len(),
                    unlock_count: unlocks.len(),
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
    use crate::{error::dto::DtoError, unlock::dto::UnlockDto};

    /// The payload type to define a value transaction.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct TransactionPayloadDto {
        #[serde(rename = "type")]
        pub kind: u32,
        pub essence: TransactionEssenceDto,
        pub unlocks: Vec<UnlockDto>,
    }

    impl From<&TransactionPayload> for TransactionPayloadDto {
        fn from(value: &TransactionPayload) -> Self {
            TransactionPayloadDto {
                kind: TransactionPayload::KIND,
                essence: value.essence().into(),
                unlocks: value.unlocks().iter().map(Into::into).collect::<Vec<_>>(),
            }
        }
    }

    impl TryFrom<&TransactionPayloadDto> for TransactionPayload {
        type Error = DtoError;

        fn try_from(value: &TransactionPayloadDto) -> Result<Self, Self::Error> {
            let mut unlocks = Vec::new();
            for b in &value.unlocks {
                unlocks.push(b.try_into()?);
            }

            Ok(TransactionPayload::new(
                (&value.essence).try_into()?,
                Unlocks::new(unlocks)?,
            )?)
        }
    }
}
