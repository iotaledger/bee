// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the milestone payload.

mod essence;
mod milestone_id;

pub use essence::MilestoneEssence;
pub(crate) use essence::PublicKeyCount;
pub use milestone_id::MilestoneId;

use crate::Error;

use bee_packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::VecPrefix,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
    Error as CryptoError,
};

use alloc::{boxed::Box, vec::Vec};
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
    signatures: VecPrefix<Box<Signature>, SignatureCount>,
}

impl MilestonePayload {
    /// The payload kind of a `MilestonePayload`.
    pub const KIND: u32 = 1;
    /// Range of allowed milestones signatures key numbers.
    pub const SIGNATURE_COUNT_RANGE: RangeInclusive<u8> = 1..=255;
    /// Length of a milestone signature.
    pub const SIGNATURE_LENGTH: usize = 64;

    /// Creates a new `MilestonePayload`.
    pub fn new(
        essence: MilestoneEssence,
        signatures: Vec<[u8; MilestonePayload::SIGNATURE_LENGTH]>,
    ) -> Result<Self, Error> {
        let signatures = VecPrefix::<Box<Signature>, SignatureCount>::try_from(
            signatures
                .into_iter()
                .map(|s| Box::new(Signature(s)))
                .collect::<Vec<_>>(),
        )
        .map_err(Error::MilestoneInvalidSignatureCount)?;

        Self::from_vec_prefix(essence, signatures)
    }

    fn from_vec_prefix(
        essence: MilestoneEssence,
        signatures: VecPrefix<Box<Signature>, SignatureCount>,
    ) -> Result<Self, Error> {
        if essence.public_keys().len() != signatures.len() {
            return Err(Error::MilestonePublicKeysSignaturesCountMismatch {
                key_count: essence.public_keys().len(),
                sig_count: signatures.len(),
            });
        };

        Ok(Self { essence, signatures })
    }

    /// Computes the identifier of a `MilestonePayload`.
    pub fn id(&self) -> MilestoneId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_to_vec());

        MilestoneId::new(hasher.finalize().into())
    }

    /// Returns the essence of a `MilestonePayload`.
    pub fn essence(&self) -> &MilestoneEssence {
        &self.essence
    }

    /// Returns the signatures of a `MilestonePayload`.
    pub fn signatures(&self) -> impl Iterator<Item = &[u8; Self::SIGNATURE_LENGTH]> + '_ {
        self.signatures.iter().map(|s| &s.0)
    }

    /// Semantically validate a `MilestonePayload`.
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
            if !applicable_public_keys.contains(&hex::encode(public_key)) {
                return Err(MilestoneValidationError::UnapplicablePublicKey(hex::encode(public_key)));
            }

            let ed25519_public_key =
                ed25519::PublicKey::try_from_bytes(*public_key).map_err(MilestoneValidationError::Crypto)?;
            let ed25519_signature = ed25519::Signature::from_bytes(signature.as_ref().0);

            if !ed25519_public_key.verify(&ed25519_signature, &essence_hash) {
                return Err(MilestoneValidationError::InvalidSignature(
                    index,
                    hex::encode(public_key),
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
        let signatures = VecPrefix::<Box<Signature>, SignatureCount>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::MilestoneInvalidSignatureCount(err.into_prefix().into()))?;

        Self::from_vec_prefix(essence, signatures).map_err(UnpackError::Packable)
    }
}
