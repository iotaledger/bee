// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::OutputId, payload::transaction::TransactionId, Error};

use derive_more::From;

use core::str::FromStr;

/// Represents an input referencing an output.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, From, packable::Packable)]
pub struct UtxoInput(OutputId);

impl UtxoInput {
    /// The input kind of a [`UtxoInput`].
    pub const KIND: u8 = 0;

    /// Creates a new [`UtxoInput`].
    pub fn new(id: TransactionId, index: u16) -> Result<Self, Error> {
        Ok(Self(OutputId::new(id, index)?))
    }

    /// Returns the output id of a [`UtxoInput`].
    pub fn output_id(&self) -> &OutputId {
        &self.0
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(UtxoInput);

impl FromStr for UtxoInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UtxoInput(OutputId::from_str(s)?))
    }
}

impl core::fmt::Display for UtxoInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for UtxoInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "UtxoInput({})", self.0)
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    /// Describes an input which references an unspent transaction output to consume.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct UtxoInputDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "transactionId")]
        pub transaction_id: String,
        #[serde(rename = "transactionOutputIndex")]
        pub transaction_output_index: u16,
    }

    impl From<&UtxoInput> for UtxoInputDto {
        fn from(value: &UtxoInput) -> Self {
            UtxoInputDto {
                kind: UtxoInput::KIND,
                transaction_id: value.output_id().transaction_id().to_string(),
                transaction_output_index: value.output_id().index(),
            }
        }
    }

    impl TryFrom<&UtxoInputDto> for UtxoInput {
        type Error = DtoError;

        fn try_from(value: &UtxoInputDto) -> Result<Self, Self::Error> {
            Ok(UtxoInput::new(
                value
                    .transaction_id
                    .parse::<TransactionId>()
                    .map_err(|_| DtoError::InvalidField("transactionId"))?,
                value.transaction_output_index,
            )?)
        }
    }
}
