// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the milestone payload.

mod essence;
mod milestone_id;

use alloc::{string::String, vec::Vec};
use core::{fmt::Debug, ops::RangeInclusive};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
    Error as CryptoError,
};
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU8, prefix::VecPrefix, Packable, PackableExt};

pub(crate) use self::essence::MilestoneMetadataLength;
pub use self::{essence::MilestoneEssence, milestone_id::MilestoneId};
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

/// A payload which defines the inclusion set of other messages in the Tangle.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
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
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_to_vec());

        MilestoneId::new(hasher.finalize().into())
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
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        error::dto::DtoError, milestone::MilestoneIndex, parent::Parents, payload::dto::PayloadDto,
        signature::dto::SignatureDto, MessageId,
    };

    /// The payload type to define a milestone.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct MilestonePayloadDto {
        #[serde(rename = "type")]
        pub kind: u32,
        pub index: u32,
        pub timestamp: u64,
        #[serde(rename = "parentMessageIds")]
        pub parents: Vec<String>,
        #[serde(rename = "inclusionMerkleProof")]
        pub inclusion_merkle_proof: String,
        #[serde(rename = "nextPoWScore")]
        pub next_pow_score: u32,
        #[serde(rename = "nextPoWScoreMilestoneIndex")]
        pub next_pow_score_milestone_index: u32,
        pub metadata: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub receipt: Option<PayloadDto>,
        pub signatures: Vec<SignatureDto>,
    }

    impl From<&MilestonePayload> for MilestonePayloadDto {
        fn from(value: &MilestonePayload) -> Self {
            MilestonePayloadDto {
                kind: MilestonePayload::KIND,
                index: *value.essence().index(),
                timestamp: value.essence().timestamp(),
                parents: value.essence().parents().iter().map(|p| p.to_string()).collect(),
                inclusion_merkle_proof: prefix_hex::encode(value.essence().merkle_proof()),
                next_pow_score: value.essence().next_pow_score(),
                next_pow_score_milestone_index: value.essence().next_pow_score_milestone_index(),
                metadata: prefix_hex::encode(value.essence().metadata()),
                receipt: value.essence().receipt().map(Into::into),
                signatures: value.signatures().iter().map(From::from).collect(),
            }
        }
    }

    impl TryFrom<&MilestonePayloadDto> for MilestonePayload {
        type Error = DtoError;

        fn try_from(value: &MilestonePayloadDto) -> Result<Self, Self::Error> {
            let essence = {
                let index = value.index;
                let timestamp = value.timestamp;
                let mut parent_ids = Vec::new();
                for msg_id in &value.parents {
                    parent_ids.push(
                        msg_id
                            .parse::<MessageId>()
                            .map_err(|_| DtoError::InvalidField("parentMessageIds"))?,
                    );
                }
                let merkle_proof = prefix_hex::decode(&value.inclusion_merkle_proof)
                    .map_err(|_| DtoError::InvalidField("inclusionMerkleProof"))?;
                let next_pow_score = value.next_pow_score;
                let next_pow_score_milestone_index = value.next_pow_score_milestone_index;
                let metadata = prefix_hex::decode(&value.metadata).map_err(|_e| DtoError::InvalidField("metadata"))?;
                let receipt = if let Some(receipt) = value.receipt.as_ref() {
                    Some(receipt.try_into()?)
                } else {
                    None
                };
                MilestoneEssence::new(
                    MilestoneIndex(index),
                    timestamp,
                    Parents::new(parent_ids)?,
                    merkle_proof,
                    next_pow_score,
                    next_pow_score_milestone_index,
                    metadata,
                    receipt,
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
