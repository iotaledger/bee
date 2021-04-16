// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the milestone payload.

mod essence;
mod milestone_id;

pub use essence::{MilestonePayloadEssence, MILESTONE_MERKLE_PROOF_LENGTH, MILESTONE_PUBLIC_KEY_LENGTH};
pub use milestone_id::{MilestoneId, MILESTONE_ID_LENGTH};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
    Error as CryptoError,
};

use alloc::{boxed::Box, vec::Vec};
use core::{convert::TryInto, ops::RangeInclusive};

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

/// A payload which defines the inclusion set of other messages in the Tangle.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestonePayload {
    essence: MilestonePayloadEssence,
    signatures: Vec<Box<[u8]>>,
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
        }

        Ok(Self {
            essence,
            signatures: vec![
                signatures
                    .iter()
                    .map(|s| std::array::IntoIter::new(*s))
                    .flatten()
                    .collect::<Vec<u8>>()
                    .into_boxed_slice(),
            ],
        })
    }

    /// Computes the identifier of a `MilestonePayload`.
    pub fn id(&self) -> MilestoneId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_new());

        MilestoneId::new(hasher.finalize().into())
    }

    /// Returns the essence of a `MilestonePayload`.
    pub fn essence(&self) -> &MilestonePayloadEssence {
        &self.essence
    }

    /// Returns the signatures of a `MilestonePayload`.
    pub fn signatures(&self) -> &Vec<Box<[u8]>> {
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
                ed25519::PublicKey::from_compressed_bytes(*public_key).map_err(MilestoneValidationError::Crypto)?;
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
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.essence.packed_len() + 0u8.packed_len() + self.signatures.len() * MILESTONE_SIGNATURE_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.essence.pack(writer)?;

        (self.signatures.len() as u8).pack(writer)?;
        for signature in &self.signatures {
            writer.write_all(&signature)?;
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let essence = MilestonePayloadEssence::unpack_inner::<R, CHECK>(reader)?;
        let signatures_len = u8::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut signatures = Vec::with_capacity(signatures_len);
        for _ in 0..signatures_len {
            signatures.push(<[u8; MILESTONE_SIGNATURE_LENGTH]>::unpack_inner::<R, CHECK>(reader)?);
        }

        Self::new(essence, signatures)
    }
}
