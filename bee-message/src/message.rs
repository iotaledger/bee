// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parents::Parents,
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    Error, MessageId,
};

use bee_common::packable::{Packable, Read, Write};
use bee_pow::providers::{miner::Miner, NonceProvider, NonceProviderBuilder};

use crypto::hashes::{blake2b::Blake2b256, Digest};

/// The minimum number of bytes in a message.
pub const MESSAGE_LENGTH_MIN: usize = 53;

/// The maximum number of bytes in a message.
pub const MESSAGE_LENGTH_MAX: usize = 32768;

const DEFAULT_POW_SCORE: f64 = 4000f64;
const DEFAULT_NONCE: u64 = 0;

/// Represent the object that nodes gossip around the network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// Specifies which network this message is meant for.
    network_id: u64,
    /// The [`MessageId`]s that this message directly approves.
    parents: Parents,
    /// The optional [Payload] of the message.
    payload: Option<Payload>,
    /// The result of the Proof of Work in order fot the message to be accepted into the tangle.
    nonce: u64,
}

impl Message {
    /// Creates a new `MessageBuilder` to construct an instance of a `Message`.
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Computes the identifier of the message.
    pub fn id(&self) -> (MessageId, Vec<u8>) {
        let bytes = self.pack_new();
        let id = Blake2b256::digest(&bytes);

        (MessageId::new(id.into()), bytes)
    }

    /// Returns the network id of a `Message`.
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Returns the parents of a `Message`.
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the optional payload of a `Message`.
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }

    /// Returns the nonce of a `Message`.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Destroys the `Message`, and returns ownership over its `Parents`.
    pub fn into_parents(self) -> Parents {
        self.parents
    }
}

impl Packable for Message {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.network_id.packed_len()
            + self.parents.packed_len()
            + option_payload_packed_len(self.payload.as_ref())
            + self.nonce.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.network_id.pack(writer)?;
        self.parents.pack(writer)?;
        option_payload_pack(writer, self.payload.as_ref())?;
        self.nonce.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let network_id = u64::unpack_inner::<R, CHECK>(reader)?;

        let parents = Parents::unpack_inner::<R, CHECK>(reader)?;

        let (payload_len, payload) = option_payload_unpack::<R, CHECK>(reader)?;

        if CHECK
            && !matches!(
                payload,
                None | Some(Payload::Transaction(_)) | Some(Payload::Milestone(_)) | Some(Payload::Indexation(_))
            )
        {
            // Safe to unwrap since it's known not to be None.
            return Err(Error::InvalidPayloadKind(payload.unwrap().kind()));
        }

        let nonce = u64::unpack_inner::<R, CHECK>(reader)?;

        // Computed instead of calling `packed_len` on Self because `payload_len` is already known and it may be
        // expensive to call `payload.packed_len()` twice.
        let message_len = network_id.packed_len() + parents.packed_len() + payload_len + nonce.packed_len();

        if CHECK && message_len > MESSAGE_LENGTH_MAX {
            return Err(Error::InvalidMessageLength(message_len));
        }

        // When parsing the message is complete, there should not be any trailing bytes left that were not parsed.
        if CHECK && reader.bytes().next().is_some() {
            return Err(Error::RemainingBytesAfterMessage);
        }

        Ok(Self {
            network_id,
            parents,
            payload,
            nonce,
        })
    }
}

/// A builder to build a `Message`.
pub struct MessageBuilder<P: NonceProvider = Miner> {
    network_id: Option<u64>,
    parents: Option<Parents>,
    payload: Option<Payload>,
    nonce_provider: Option<(P, f64)>,
}

impl<P: NonceProvider> Default for MessageBuilder<P> {
    fn default() -> Self {
        Self {
            network_id: None,
            parents: None,
            payload: None,
            nonce_provider: None,
        }
    }
}

impl<P: NonceProvider> MessageBuilder<P> {
    /// Creates a new `MessageBuilder`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a network id to a `MessageBuilder`.
    pub fn with_network_id(mut self, network_id: u64) -> Self {
        self.network_id = Some(network_id);
        self
    }

    /// Adds parents to a `MessageBuilder`.
    pub fn with_parents(mut self, parents: Parents) -> Self {
        self.parents = Some(parents);
        self
    }

    /// Adds a payload to a `MessageBuilder`.
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Adds a nonce provider to a `MessageBuilder`.
    pub fn with_nonce_provider(mut self, nonce_provider: P, target_score: f64) -> Self {
        self.nonce_provider = Some((nonce_provider, target_score));
        self
    }

    /// Finishes the `MessageBuilder` into a `Message`.
    pub fn finish(self) -> Result<Message, Error> {
        let network_id = self.network_id.ok_or(Error::MissingField("network_id"))?;
        let parents = self.parents.ok_or(Error::MissingField("parents"))?;

        if !matches!(
            self.payload,
            None | Some(Payload::Transaction(_)) | Some(Payload::Milestone(_)) | Some(Payload::Indexation(_))
        ) {
            // Safe to unwrap since it's known not to be None.
            return Err(Error::InvalidPayloadKind(self.payload.unwrap().kind()));
        }

        let mut message = Message {
            network_id,
            parents,
            payload: self.payload,
            nonce: 0,
        };

        let message_bytes = message.pack_new();

        if message_bytes.len() > MESSAGE_LENGTH_MAX {
            return Err(Error::InvalidMessageLength(message_bytes.len()));
        }

        let (nonce_provider, target_score) = self
            .nonce_provider
            .unwrap_or((P::Builder::new().finish(), DEFAULT_POW_SCORE));

        message.nonce = nonce_provider
            .nonce(
                &message_bytes[..message_bytes.len() - std::mem::size_of::<u64>()],
                target_score,
            )
            .unwrap_or(DEFAULT_NONCE);

        Ok(message)
    }
}
