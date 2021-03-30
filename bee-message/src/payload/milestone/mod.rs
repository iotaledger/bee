// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod essence;
mod milestone_id;

pub use essence::{MilestonePayloadEssence, MILESTONE_MERKLE_PROOF_LENGTH, MILESTONE_PUBLIC_KEY_LENGTH};
pub use milestone_id::{MilestoneId, MILESTONE_ID_LENGTH};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
};

use alloc::{boxed::Box, vec::Vec};
use core::{convert::TryInto, ops::RangeInclusive};

pub const MILESTONE_SIGNATURE_COUNT_RANGE: RangeInclusive<usize> = 1..=255;
pub const MILESTONE_SIGNATURE_LENGTH: usize = 64;

#[derive(Debug)]
pub enum MilestoneValidationError {
    InvalidMinThreshold,
    TooFewSignatures(usize, usize),
    InsufficientApplicablePublicKeys(usize, usize),
    UnapplicablePublicKey(String),
    InvalidSignature(usize, String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestonePayload {
    essence: MilestonePayloadEssence,
    // TODO length is 64, change to array when std::array::LengthAtMost32 disappears.
    signatures: Vec<Box<[u8]>>,
}

impl MilestonePayload {
    pub const KIND: u32 = 1;

    pub fn new(essence: MilestonePayloadEssence, signatures: Vec<Box<[u8]>>) -> Result<Self, Error> {
        if !MILESTONE_SIGNATURE_COUNT_RANGE.contains(&signatures.len()) {
            return Err(Error::MilestoneInvalidSignatureCount(signatures.len()));
        }

        if essence.public_keys().len() != signatures.len() {
            return Err(Error::MilestonePublicKeysSignaturesCountMismatch(
                essence.public_keys().len(),
                signatures.len(),
            ));
        }

        // TODO check signature length

        Ok(Self { essence, signatures })
    }

    pub fn id(&self) -> MilestoneId {
        let mut hasher = Blake2b256::new();

        hasher.update(Self::KIND.to_le_bytes());
        hasher.update(self.pack_new());

        MilestoneId::new(hasher.finalize().into())
    }

    pub fn essence(&self) -> &MilestonePayloadEssence {
        &self.essence
    }

    pub fn signatures(&self) -> &Vec<Box<[u8]>> {
        &self.signatures
    }

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
            // TODO use concrete ED25 types
            if !applicable_public_keys.contains(&hex::encode(public_key)) {
                return Err(MilestoneValidationError::UnapplicablePublicKey(hex::encode(public_key)));
            }

            // TODO unwrap
            let ed25519_public_key = ed25519::PublicKey::from_compressed_bytes(*public_key).unwrap();
            // TODO unwrap
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let essence = MilestonePayloadEssence::unpack(reader)?;
        let signatures_len = u8::unpack(reader)? as usize;
        let mut signatures = Vec::with_capacity(signatures_len);
        for _ in 0..signatures_len {
            let mut signature = vec![0u8; MILESTONE_SIGNATURE_LENGTH];
            reader.read_exact(&mut signature)?;
            signatures.push(signature.into_boxed_slice());
        }

        Self::new(essence, signatures)
    }
}
