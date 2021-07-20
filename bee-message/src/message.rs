// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{parents::Parents, payload::Payload, MessageId, MessagePackError, MessageUnpackError, ValidationError};

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
};

use alloc::vec::Vec;
use core::{convert::TryInto, ops::RangeInclusive};

/// Range (in bytes) of a valid message length.
pub const MESSAGE_LENGTH_RANGE: RangeInclusive<usize> = 53..=32768;

/// Length (in bytes) of a public key.
pub const MESSAGE_PUBLIC_KEY_LENGTH: usize = 32;

/// Length (in bytes) of a message signature.
pub const MESSAGE_SIGNATURE_LENGTH: usize = 64;

/// Represents the object that nodes gossip around the network.
///
/// `Message`s must:
/// * Have a length (in bytes) within `MESSAGE_LENGTH_RANGE`.
/// * Ensure all applicable data is appropriately validated.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "enable-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// Message [Parents].
    pub(crate) parents: Parents,
    /// The public key of the issuing node.
    pub(crate) issuer_public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH],
    /// The Unix timestamp at the moment of issue.
    pub(crate) issue_timestamp: u64,
    /// The sequence number of the message, indicating the marker sequence it belongs to.
    pub(crate) sequence_number: u32,
    /// The optional [Payload] of the message.
    pub(crate) payload: Option<Payload>,
    /// The result of the Proof of Work in order for the message to be accepted into the tangle.
    pub(crate) nonce: u64,
    /// Signature signing the above message fields.
    #[cfg_attr(feature = "enable-serde", serde(with = "serde_big_array::BigArray"))]
    pub(crate) signature: [u8; MESSAGE_SIGNATURE_LENGTH],
}

impl Message {
    /// Computes the identifier of the message.
    pub fn id(&self) -> (MessageId, Vec<u8>) {
        let bytes = self.pack_to_vec().unwrap();

        let id = Blake2b256::digest(&bytes);

        (MessageId::new(id.into()), bytes)
    }

    /// Returns the parents of a `Message`.
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the `Message` issuer public key.
    pub fn issuer_public_key(&self) -> &[u8; MESSAGE_PUBLIC_KEY_LENGTH] {
        &self.issuer_public_key
    }

    /// Returns the `Message` issuance timestamp.
    pub fn issue_timestamp(&self) -> u64 {
        self.issue_timestamp
    }

    /// Returns the sequence number of a `Message`.
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    /// Returns the optional payload of a `Message`.
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }

    /// Returns the nonce of a `Message`.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Returns the `Message` signature.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Hashes the `Message` contents, excluding the signature.
    pub fn hash(&self) -> [u8; 32] {
        let mut bytes = self.pack_to_vec().unwrap();

        bytes = bytes[..bytes.len() - core::mem::size_of::<u64>()].to_vec();

        Blake2b256::digest(&bytes).into()
    }

    /// Verifies the `Message` signature against the contents of the `Message`.
    pub fn verify(&self) -> Result<(), ValidationError> {
        let ed25519_public_key = ed25519::PublicKey::from_compressed_bytes(self.issuer_public_key)?;

        // Unwrapping is okay here, since the length of the signature is already known to be correct.
        let ed25519_signature = ed25519::Signature::from_bytes(self.signature.to_vec().try_into().unwrap());

        let hash = self.hash();

        if !ed25519_public_key.verify(&ed25519_signature, &hash) {
            Err(ValidationError::InvalidSignature)
        } else {
            Ok(())
        }
    }
}

impl Packable for Message {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.parents.packed_len()
            + self.issuer_public_key.packed_len()
            + self.issue_timestamp.packed_len()
            + self.sequence_number.packed_len()
            + self.payload.packed_len()
            + self.nonce.packed_len()
            + self.signature.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.parents.pack(packer)?;
        self.issuer_public_key.pack(packer).map_err(PackError::infallible)?;
        self.issue_timestamp.pack(packer).map_err(PackError::infallible)?;
        self.sequence_number.pack(packer).map_err(PackError::infallible)?;
        self.payload.pack(packer)?;
        self.nonce.pack(packer).map_err(PackError::infallible)?;
        self.signature.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let parents = Parents::unpack(unpacker)?;
        let issuer_public_key = <[u8; MESSAGE_PUBLIC_KEY_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;
        let issue_timestamp = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let sequence_number = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let payload = Option::<Payload>::unpack(unpacker).map_err(UnpackError::coerce)?;
        let nonce = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let signature = <[u8; MESSAGE_SIGNATURE_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;

        let message = Self {
            parents,
            issuer_public_key,
            issue_timestamp,
            sequence_number,
            payload,
            nonce,
            signature,
        };

        let len = message.pack_to_vec().unwrap().len();
        validate_message_len(len).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(message)
    }
}

pub(crate) fn validate_message_len(len: usize) -> Result<(), ValidationError> {
    if !MESSAGE_LENGTH_RANGE.contains(&len) {
        Err(ValidationError::InvalidMessageLength(len))
    } else {
        Ok(())
    }
}
