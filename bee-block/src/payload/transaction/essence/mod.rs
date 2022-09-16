// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod regular;

use crypto::hashes::{blake2b::Blake2b256, Digest};
use derive_more::From;
use packable::PackableExt;

pub(crate) use self::regular::{InputCount, OutputCount};
pub use self::regular::{RegularTransactionEssence, RegularTransactionEssenceBuilder};
use crate::Error;

/// A generic essence that can represent different types defining transaction essences.
#[derive(Clone, Debug, Eq, PartialEq, From, packable::Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidEssenceKind)]
pub enum TransactionEssence {
    /// A regular transaction essence.
    #[packable(tag = RegularTransactionEssence::KIND)]
    Regular(RegularTransactionEssence),
}

impl TransactionEssence {
    /// Returns the essence kind of an [`TransactionEssence`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Regular(_) => RegularTransactionEssence::KIND,
        }
    }

    /// Return the Blake2b hash of an [`TransactionEssence`].
    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_to_vec()).into()
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    pub use super::regular::dto::RegularTransactionEssenceDto;
    use super::*;
    use crate::{error::dto::DtoError, protocol::ProtocolParameters};

    /// Describes all the different essence types.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum TransactionEssenceDto {
        Regular(RegularTransactionEssenceDto),
    }

    impl From<&TransactionEssence> for TransactionEssenceDto {
        fn from(value: &TransactionEssence) -> Self {
            match value {
                TransactionEssence::Regular(r) => TransactionEssenceDto::Regular(r.into()),
            }
        }
    }

    impl TransactionEssence {
        pub fn try_from_dto(
            value: &TransactionEssenceDto,
            protocol_parameters: &ProtocolParameters,
        ) -> Result<TransactionEssence, DtoError> {
            match value {
                TransactionEssenceDto::Regular(r) => Ok(TransactionEssence::Regular(
                    RegularTransactionEssence::try_from_dto(r, protocol_parameters)?,
                )),
            }
        }
    }
}
