// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parents::Parents,
    payload::{OptionalPayload, Payload},
    MessageId, MessageUnpackError, ValidationError,
};

use bee_packable::{Packable, Packer, UnpackError, UnpackErrorExt, Unpacker};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
};

use core::ops::RangeInclusive;

/// Range (in bytes) of a valid message length.
/// The maximum length is 64KB, and minimum length is calculated from message containing an empty data payload and two
/// parents.
pub const MESSAGE_LENGTH_RANGE: RangeInclusive<usize> = 123..=65536;

/// Length (in bytes) of a public key.
pub const MESSAGE_PUBLIC_KEY_LENGTH: usize = 32;

/// Length (in bytes) of a message signature.
pub const MESSAGE_SIGNATURE_LENGTH: usize = 64;

/// Messages are of version 1.
const MESSAGE_VERSION: u8 = 1;

/// Represents the object that nodes gossip around the network.
///
/// [`Message`]s must:
/// * Have a length (in bytes) within [`MESSAGE_LENGTH_RANGE`].
/// * Ensure all applicable data is appropriately validated.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// List of parent [`MessageId`]s.
    pub(crate) parents: Parents,
    /// The public key of the issuing node.
    pub(crate) issuer_public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH],
    /// The Unix timestamp at the moment of issue.
    pub(crate) issue_timestamp: u64,
    /// The sequence number of the message, indicating the marker sequence it belongs to.
    pub(crate) sequence_number: u32,
    /// The optional [Payload] of the message.
    pub(crate) payload: OptionalPayload,
    /// The result of the Proof of Work in order for the message to be accepted into the tangle.
    pub(crate) nonce: u64,
    /// Signature signing the above message fields.
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    pub(crate) signature: [u8; MESSAGE_SIGNATURE_LENGTH],
}

impl Message {
    /// Computes the identifier of the message.
    pub fn id(&self) -> MessageId {
        let bytes = self.pack_to_vec();

        let id = Blake2b256::digest(&bytes);

        MessageId::new(id.into())
    }

    /// Returns the parents of a [`Message`].
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the [`Message`] issuer public key.
    pub fn issuer_public_key(&self) -> &[u8; MESSAGE_PUBLIC_KEY_LENGTH] {
        &self.issuer_public_key
    }

    /// Returns the [`Message`] issuance timestamp.
    pub fn issue_timestamp(&self) -> u64 {
        self.issue_timestamp
    }

    /// Returns the sequence number of a [`Message`].
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    /// Returns the optional payload of a [`Message`].
    pub fn payload(&self) -> Option<&Payload> {
        match &self.payload {
            OptionalPayload::None => None,
            OptionalPayload::Some(payload) => Some(payload),
        }
    }

    /// Returns the nonce of a [`Message`].
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Returns the [`Message`] signature.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Hashes the [`Message`] contents, excluding the signature.
    pub fn hash(&self) -> [u8; 32] {
        let mut bytes = self.pack_to_vec();

        bytes = bytes[..bytes.len() - core::mem::size_of::<u64>()].to_vec();

        Blake2b256::digest(&bytes).into()
    }

    /// Verifies the [`Message`] signature against the contents of the [`Message`].
    pub fn verify(&self) -> Result<(), ValidationError> {
        let ed25519_public_key = ed25519::PublicKey::try_from_bytes(self.issuer_public_key)?;

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
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        MESSAGE_VERSION.packed_len()
            + self.parents.packed_len()
            + self.issuer_public_key.packed_len()
            + self.issue_timestamp.packed_len()
            + self.sequence_number.packed_len()
            + self.payload.packed_len()
            + self.nonce.packed_len()
            + self.signature.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        MESSAGE_VERSION.pack(packer)?;
        self.parents.pack(packer)?;
        self.issuer_public_key.pack(packer)?;
        self.issue_timestamp.pack(packer)?;
        self.sequence_number.pack(packer)?;
        self.payload.pack(packer)?;
        self.nonce.pack(packer)?;
        self.signature.pack(packer)
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).infallible()?;
        validate_message_version(version).map_err(UnpackError::from_packable)?;

        let parents = Parents::unpack(unpacker)?;
        let issuer_public_key = <[u8; MESSAGE_PUBLIC_KEY_LENGTH]>::unpack(unpacker).infallible()?;
        let issue_timestamp = u64::unpack(unpacker).infallible()?;
        let sequence_number = u32::unpack(unpacker).infallible()?;
        let payload = OptionalPayload::unpack(unpacker)?;
        let nonce = u64::unpack(unpacker).infallible()?;
        let signature = <[u8; MESSAGE_SIGNATURE_LENGTH]>::unpack(unpacker).infallible()?;

        let message = Self {
            parents,
            issuer_public_key,
            issue_timestamp,
            sequence_number,
            payload,
            nonce,
            signature,
        };

        let len = message.pack_to_vec().len();
        validate_message_len(len).map_err(UnpackError::from_packable)?;

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

fn validate_message_version(version: u8) -> Result<(), ValidationError> {
    if version != MESSAGE_VERSION {
        Err(ValidationError::InvalidMessageVersion(version))
    } else {
        Ok(())
    }
}
