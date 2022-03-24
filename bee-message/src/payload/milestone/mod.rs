// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the milestone payload.

mod essence;
mod milestone_id;

pub use essence::MilestoneEssence;
pub(crate) use essence::PublicKeyCount;
pub use milestone_id::MilestoneId;

use crate::Error;

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
    Error as CryptoError,
};
use packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::VecPrefix,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use alloc::{string::String, vec::Vec};
use core::{fmt::Debug, ops::RangeInclusive};

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

#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
struct Signature(
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))] [u8; MilestonePayload::SIGNATURE_LENGTH],
);

pub(crate) type SignatureCount =
    BoundedU8<{ *MilestonePayload::SIGNATURE_COUNT_RANGE.start() }, { *MilestonePayload::SIGNATURE_COUNT_RANGE.end() }>;

/// A payload which defines the inclusion set of other messages in the Tangle.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestonePayload {
    essence: MilestoneEssence,
    signatures: VecPrefix<Signature, SignatureCount>,
}

impl MilestonePayload {
    /// The payload kind of a [`MilestonePayload`].
    pub const KIND: u32 = 1;
    /// Range of allowed milestones signatures key numbers.
    pub const SIGNATURE_COUNT_RANGE: RangeInclusive<u8> = 1..=255;
    /// Length of a milestone signature.
    pub const SIGNATURE_LENGTH: usize = 64;

    /// Creates a new [`MilestonePayload`].
    pub fn new(
        essence: MilestoneEssence,
        signatures: Vec<[u8; MilestonePayload::SIGNATURE_LENGTH]>,
    ) -> Result<Self, Error> {
        let signatures = VecPrefix::<Signature, SignatureCount>::try_from(
            signatures.into_iter().map(Signature).collect::<Vec<Signature>>(),
        )
        .map_err(Error::MilestoneInvalidSignatureCount)?;

        verify_essence_signatures(&essence, &signatures)?;

        Ok(Self { essence, signatures })
    }

    /// Returns the essence of a [`MilestonePayload`].
    pub fn essence(&self) -> &MilestoneEssence {
        &self.essence
    }

    /// Returns the signatures of a [`MilestonePayload`].
    pub fn signatures(&self) -> impl Iterator<Item = &[u8; Self::SIGNATURE_LENGTH]> + '_ {
        self.signatures.iter().map(|s| &s.0)
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

        for (index, (public_key, signature)) in self
            .essence()
            .public_keys()
            .iter()
            .zip(self.signatures.iter())
            .enumerate()
        {
            if !applicable_public_keys.contains(&prefix_hex::encode(public_key)) {
                return Err(MilestoneValidationError::UnapplicablePublicKey(prefix_hex::encode(
                    *public_key,
                )));
            }

            let ed25519_public_key =
                ed25519::PublicKey::try_from_bytes(*public_key).map_err(MilestoneValidationError::Crypto)?;
            let ed25519_signature = ed25519::Signature::from_bytes(signature.0);

            if !ed25519_public_key.verify(&ed25519_signature, &essence_hash) {
                return Err(MilestoneValidationError::InvalidSignature(
                    index,
                    prefix_hex::encode(public_key),
                ));
            }
        }

        Ok(())
    }
}

impl Packable for MilestonePayload {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.essence.pack(packer)?;
        self.signatures.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let essence = MilestoneEssence::unpack::<_, VERIFY>(unpacker)?;
        let signatures = VecPrefix::<Signature, SignatureCount>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::MilestoneInvalidSignatureCount(err.into_prefix_err().into()))?;

        if VERIFY {
            verify_essence_signatures(&essence, &signatures).map_err(UnpackError::Packable)?;
        }

        Ok(Self { essence, signatures })
    }
}

fn verify_essence_signatures(essence: &MilestoneEssence, signatures: &[Signature]) -> Result<(), Error> {
    if essence.public_keys().len() != signatures.len() {
        Err(Error::MilestonePublicKeysSignaturesCountMismatch {
            key_count: essence.public_keys().len(),
            sig_count: signatures.len(),
        })
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
        error::dto::DtoError, milestone::MilestoneIndex, parent::Parents, payload::dto::PayloadDto, MessageId,
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
        #[serde(rename = "publicKeys")]
        pub public_keys: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub receipt: Option<PayloadDto>,
        pub signatures: Vec<String>,
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
                public_keys: value.essence().public_keys().iter().map(prefix_hex::encode).collect(),
                receipt: value.essence().receipt().map(Into::into),
                signatures: value.signatures().map(prefix_hex::encode).collect(),
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
                let mut public_keys = Vec::new();
                for v in &value.public_keys {
                    public_keys.push(prefix_hex::decode(v).map_err(|_| DtoError::InvalidField("publicKeys"))?);
                }
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
                    public_keys,
                    receipt,
                )?
            };
            let mut signatures = Vec::new();
            for v in &value.signatures {
                let sig: Vec<u8> = prefix_hex::decode(v).map_err(|_| DtoError::InvalidField("signatures"))?;
                signatures.push(sig.try_into().map_err(|_| DtoError::InvalidField("signatures"))?)
            }

            Ok(MilestonePayload::new(essence, signatures)?)
        }
    }
}
