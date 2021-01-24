// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod essence;

pub use essence::{MilestonePayloadEssence, MILESTONE_MERKLE_PROOF_LENGTH, MILESTONE_PUBLIC_KEY_LENGTH};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use crypto::ed25519;
use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, vec::Vec};
use core::convert::TryInto;

pub(crate) const MILESTONE_PAYLOAD_TYPE: u32 = 1;

pub const MILESTONE_SIGNATURE_LENGTH: usize = 64;

#[derive(Debug)]
pub enum MilestoneValidationError {
    InvalidMinThreshold,
    TooFewSignatures(usize, usize),
    SignaturesPublicKeysCountMismatch(usize, usize),
    InsufficientApplicablePublicKeys(usize, usize),
    UnapplicablePublicKey(String),
    InvalidSignature(usize, String),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MilestonePayload {
    essence: MilestonePayloadEssence,
    // TODO length is 64, change to array when std::array::LengthAtMost32 disappears.
    signatures: Vec<Box<[u8]>>,
}

impl MilestonePayload {
    pub fn new(essence: MilestonePayloadEssence, signatures: Vec<Box<[u8]>>) -> Self {
        Self { essence, signatures }
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

        if self.signatures().is_empty() || self.signatures().len() < min_threshold {
            return Err(MilestoneValidationError::TooFewSignatures(
                min_threshold,
                self.signatures().len(),
            ));
        }

        // TODO move this check to the build/unpack validation
        if self.signatures().len() != self.essence().public_keys().len() {
            return Err(MilestoneValidationError::SignaturesPublicKeysCountMismatch(
                self.signatures().len(),
                self.essence().public_keys().len(),
            ));
        }

        let essence_bytes = self.essence().pack_new();

        // TODO zip public key and signature
        for (index, public_key) in self.essence().public_keys().iter().enumerate() {
            // TODO use concrete ED25 types
            if !applicable_public_keys.contains(&hex::encode(public_key)) {
                return Err(MilestoneValidationError::UnapplicablePublicKey(hex::encode(public_key)));
            }

            // TODO unwrap
            let ed25519_public_key = ed25519::PublicKey::from_compressed_bytes(*public_key).unwrap();
            // TODO unwrap
            let ed25519_signature =
                ed25519::Signature::from_bytes(self.signatures()[index].as_ref().try_into().unwrap());

            if !ed25519::verify(&ed25519_public_key, &ed25519_signature, &essence_bytes) {
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

        Ok(Self { essence, signatures })
    }
}
