// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the milestone payload.

mod essence;
mod milestone_id;

pub use essence::{MilestonePayloadEssence, MILESTONE_MERKLE_PROOF_LENGTH, MILESTONE_PUBLIC_KEY_LENGTH};
pub use milestone_id::{MilestoneId, MILESTONE_ID_LENGTH};

use crate::Error;

use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
    Error as CryptoError,
};

use alloc::{boxed::Box, vec::Vec};
use core::{
    convert::Infallible,
    ops::{Deref, RangeInclusive},
};

/// Range of allowed milestones signatures key numbers.
pub const MILESTONE_SIGNATURE_COUNT_RANGE: RangeInclusive<usize> = 1..=255;
/// Length of a milestone signature.
pub const MILESTONE_SIGNATURE_LENGTH: usize = 64;

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
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))] [u8; MILESTONE_SIGNATURE_LENGTH],
);

impl Deref for Signature {
    type Target = [u8; MILESTONE_SIGNATURE_LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A payload which defines the inclusion set of other messages in the Tangle.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestonePayload {
    essence: MilestonePayloadEssence,
    signatures: Vec<Signature>,
}

impl MilestonePayload {
    /// The payload kind of a `MilestonePayload`.
    pub const KIND: u32 = 1;

    /// Creates a new `MilestonePayload`.
    pub fn new(
        essence: MilestonePayloadEssence,
        signatures: Vec<[u8; MILESTONE_SIGNATURE_LENGTH]>,
    ) -> Result<Self, Error> {
        if !MILESTONE_SIGNATURE_COUNT_RANGE.contains(&signatures.len()) {
            return Err(Error::MilestoneInvalidSignatureCount(signatures.len()));
        }

        if essence.public_keys().len() != signatures.len() {
            return Err(Error::MilestonePublicKeysSignaturesCountMismatch(
                essence.public_keys().len(),
                signatures.len(),
            ));
        };

        Ok(Self {
            essence,
            signatures: signatures.into_iter().map(Signature).collect(),
        })
    }

    /// Computes the identifier of a `MilestonePayload`.
    pub fn id(&self) -> MilestoneId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_to_vec().unwrap());

        MilestoneId::new(hasher.finalize().into())
    }

    /// Returns the essence of a `MilestonePayload`.
    pub fn essence(&self) -> &MilestonePayloadEssence {
        &self.essence
    }

    /// Returns the signatures of a `MilestonePayload`.
    pub fn signatures(&self) -> &Vec<impl Deref<Target = [u8; MILESTONE_SIGNATURE_LENGTH]>> {
        &self.signatures
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

        if self.signatures().len() < min_threshold {
            return Err(MilestoneValidationError::TooFewSignatures(
                min_threshold,
                self.signatures().len(),
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
            // This unwrap is fine as the length of the signature has already been verified.
            let ed25519_signature = ed25519::Signature::from_bytes(signature.as_ref().try_into().unwrap());

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

        (self.signatures.len() as u8).pack(packer)?;
        for signature in &self.signatures {
            signature.pack(packer).infallible()?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let essence = MilestonePayloadEssence::unpack::<_, VERIFY>(unpacker)?;
        let signatures_len = u8::unpack::<_, VERIFY>(unpacker).infallible()? as usize;
        let mut signatures = Vec::with_capacity(signatures_len);
        for _ in 0..signatures_len {
            signatures.push(<[u8; MILESTONE_SIGNATURE_LENGTH]>::unpack::<_, VERIFY>(unpacker).infallible()?);
        }

        Self::new(essence, signatures).map_err(UnpackError::Packable)
    }
}
