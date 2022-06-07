// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the milestone payload.

mod essence;
mod index;
mod merkle;
mod milestone_id;

///
pub mod option;

use alloc::{string::String, vec::Vec};
use core::{fmt::Debug, ops::RangeInclusive};

use crypto::{signatures::ed25519, Error as CryptoError};
use iterator_sorted::is_unique_sorted;
pub(crate) use option::{MigratedFundsAmount, MilestoneOptionCount, ReceiptFundsCount};
use packable::{bounded::BoundedU8, prefix::VecPrefix, Packable};

pub use self::{
    essence::MilestoneEssence,
    index::MilestoneIndex,
    merkle::MerkleRoot,
    milestone_id::MilestoneId,
    option::{MilestoneOption, MilestoneOptions, ParametersMilestoneOption, ReceiptMilestoneOption},
};
pub(crate) use self::{essence::MilestoneMetadataLength, option::BinaryParametersLength};
use crate::{signature::Signature, Error};

#[derive(Debug)]
#[allow(missing_docs)]
pub enum MilestoneValidationError {
    InvalidMinThreshold,
    TooFewSignatures(usize, usize),
    InsufficientApplicablePublicKeys(usize, usize),
    UnapplicablePublicKey(String),
    InvalidSignature(usize, String),
    Crypto(CryptoError),
}

impl From<CryptoError> for MilestoneValidationError {
    fn from(error: CryptoError) -> Self {
        MilestoneValidationError::Crypto(error)
    }
}

pub(crate) type SignatureCount =
    BoundedU8<{ *MilestonePayload::SIGNATURE_COUNT_RANGE.start() }, { *MilestonePayload::SIGNATURE_COUNT_RANGE.end() }>;

/// A payload which defines the inclusion set of other blocks in the Tangle.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct MilestonePayload {
    essence: MilestoneEssence,
    #[packable(verify_with = verify_signatures)]
    #[packable(unpack_error_with = |e| e.unwrap_item_err_or_else(|p| Error::MilestoneInvalidSignatureCount(p.into())))]
    signatures: VecPrefix<Signature, SignatureCount>,
}

impl MilestonePayload {
    /// The payload kind of a [`MilestonePayload`].
    pub const KIND: u32 = 7;
    /// Range of allowed milestones signatures key numbers.
    pub const SIGNATURE_COUNT_RANGE: RangeInclusive<u8> = 1..=255;
    /// Length of a milestone signature.
    pub const SIGNATURE_LENGTH: usize = 64;

    /// Creates a new [`MilestonePayload`].
    pub fn new(essence: MilestoneEssence, signatures: Vec<Signature>) -> Result<Self, Error> {
        let signatures = VecPrefix::<Signature, SignatureCount>::try_from(signatures)
            .map_err(Error::MilestoneInvalidSignatureCount)?;

        Ok(Self { essence, signatures })
    }

    /// Returns the essence of a [`MilestonePayload`].
    pub fn essence(&self) -> &MilestoneEssence {
        &self.essence
    }

    /// Returns the signatures of a [`MilestonePayload`].
    pub fn signatures(&self) -> &[Signature] {
        &self.signatures
    }

    /// Computes the identifier of a [`MilestonePayload`].
    pub fn id(&self) -> MilestoneId {
        MilestoneId::new(self.essence().hash())
    }

    /// Semantically validate a [`MilestonePayload`].
    pub fn validate(
        &self,
        applicable_public_keys: &[String],
        min_threshold: usize,
    ) -> Result<(), MilestoneValidationError> {
        if min_threshold == 0 {
            return Err(MilestoneValidationError::InvalidMinThreshold);
        }

        if applicable_public_keys.len() < min_threshold {
            return Err(MilestoneValidationError::InsufficientApplicablePublicKeys(
                applicable_public_keys.len(),
                min_threshold,
            ));
        }

        if self.signatures.len() < min_threshold {
            return Err(MilestoneValidationError::TooFewSignatures(
                min_threshold,
                self.signatures.len(),
            ));
        }

        let essence_hash = self.essence().hash();

        for (index, signature) in self.signatures().iter().enumerate() {
            let Signature::Ed25519(signature) = signature;

            if !applicable_public_keys.contains(&hex::encode(signature.public_key())) {
                return Err(MilestoneValidationError::UnapplicablePublicKey(prefix_hex::encode(
                    *signature.public_key(),
                )));
            }

            let ed25519_public_key = ed25519::PublicKey::try_from_bytes(*signature.public_key())
                .map_err(MilestoneValidationError::Crypto)?;
            let ed25519_signature = ed25519::Signature::from_bytes(*signature.signature());

            if !ed25519_public_key.verify(&ed25519_signature, &essence_hash) {
                return Err(MilestoneValidationError::InvalidSignature(
                    index,
                    prefix_hex::encode(signature.public_key()),
                ));
            }
        }

        Ok(())
    }
}

fn verify_signatures<const VERIFY: bool>(signatures: &[Signature]) -> Result<(), Error> {
    if VERIFY
        && !is_unique_sorted(signatures.iter().map(|signature| {
            let Signature::Ed25519(signature) = signature;
            signature.public_key()
        }))
    {
        Err(Error::MilestoneSignaturesNotUniqueSorted)
    } else {
        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use core::str::FromStr;

    use serde::{Deserialize, Serialize};

    use self::option::dto::MilestoneOptionDto;
    use super::*;
    use crate::{
        constant::PROTOCOL_VERSION, error::dto::DtoError, parent::Parents, payload::milestone::MilestoneIndex,
        signature::dto::SignatureDto, BlockId,
    };

    /// The payload type to define a milestone.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct MilestonePayloadDto {
        #[serde(rename = "type")]
        pub kind: u32,
        pub index: u32,
        pub timestamp: u32,
        #[serde(rename = "protocolVersion")]
        pub protocol_version: u8,
        #[serde(rename = "previousMilestoneId")]
        pub previous_milestone_id: String,
        pub parents: Vec<String>,
        #[serde(rename = "inclusionMerkleRoot")]
        pub inclusion_merkle_root: String,
        #[serde(rename = "appliedMerkleRoot")]
        pub applied_merkle_root: String,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub options: Vec<MilestoneOptionDto>,
        #[serde(skip_serializing_if = "String::is_empty", default)]
        pub metadata: String,
        pub signatures: Vec<SignatureDto>,
    }

    impl From<&MilestonePayload> for MilestonePayloadDto {
        fn from(value: &MilestonePayload) -> Self {
            MilestonePayloadDto {
                kind: MilestonePayload::KIND,
                index: *value.essence().index(),
                timestamp: value.essence().timestamp(),
                protocol_version: value.essence().protocol_version(),
                previous_milestone_id: value.essence().previous_milestone_id().to_string(),
                parents: value.essence().parents().iter().map(|p| p.to_string()).collect(),
                inclusion_merkle_root: value.essence().inclusion_merkle_root().to_string(),
                applied_merkle_root: value.essence().applied_merkle_root().to_string(),
                metadata: prefix_hex::encode(value.essence().metadata()),
                options: value.essence().options().iter().map(Into::into).collect::<_>(),
                signatures: value.signatures().iter().map(From::from).collect(),
            }
        }
    }

    impl TryFrom<&MilestonePayloadDto> for MilestonePayload {
        type Error = DtoError;

        fn try_from(value: &MilestonePayloadDto) -> Result<Self, Self::Error> {
            if value.protocol_version != PROTOCOL_VERSION {
                return Err(Error::ProtocolVersionMismatch {
                    expected: PROTOCOL_VERSION,
                    actual: value.protocol_version,
                }
                .into());
            }

            let essence = {
                let index = value.index;

                let timestamp = value.timestamp;

                let previous_milestone_id = MilestoneId::from_str(&value.previous_milestone_id)
                    .map_err(|_| DtoError::InvalidField("lastMilestoneId"))?;

                let mut parent_ids = Vec::new();

                for block_id in &value.parents {
                    parent_ids.push(
                        block_id
                            .parse::<BlockId>()
                            .map_err(|_| DtoError::InvalidField("parents"))?,
                    );
                }

                let inclusion_merkle_root = MerkleRoot::from_str(&value.inclusion_merkle_root)
                    .map_err(|_| DtoError::InvalidField("inclusionMerkleRoot"))?;

                let applied_merkle_root = MerkleRoot::from_str(&value.applied_merkle_root)
                    .map_err(|_| DtoError::InvalidField("appliedMerkleRoot"))?;

                let options = MilestoneOptions::try_from(
                    value
                        .options
                        .iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()?,
                )?;

                let metadata = if !value.metadata.is_empty() {
                    prefix_hex::decode(&value.metadata).map_err(|_| DtoError::InvalidField("metadata"))?
                } else {
                    Vec::new()
                };

                MilestoneEssence::new(
                    MilestoneIndex(index),
                    timestamp,
                    previous_milestone_id,
                    Parents::new(parent_ids)?,
                    inclusion_merkle_root,
                    applied_merkle_root,
                    metadata,
                    options,
                )?
            };

            let mut signatures = Vec::new();
            for v in &value.signatures {
                signatures.push(v.try_into().map_err(|_| DtoError::InvalidField("signatures"))?)
            }

            Ok(MilestonePayload::new(essence, signatures)?)
        }
    }
}
