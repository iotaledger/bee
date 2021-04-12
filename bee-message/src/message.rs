// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    Error, MessageId, Parents,
};

use bee_common::packable::{Packable, Read, Write};
use bee_pow::providers::{Miner, Provider, ProviderBuilder};

use crypto::hashes::{blake2b::Blake2b256, Digest};

use std::sync::{atomic::AtomicBool, Arc};

/// The minimum number of bytes in a message.
pub const MESSAGE_LENGTH_MIN: usize = 53;

/// The maximum number of bytes in a message (1024*32).
/// <https://github.com/GalRogozinski/protocol-rfcs/blob/message/text/0017-message/0017-message.md#message-validation>
pub const MESSAGE_LENGTH_MAX: usize = 32768;

/// A `Message` is the object that nodes gossip around the network.
///
/// Spec: #iota-protocol-rfc-draft
/// <https://github.com/GalRogozinski/protocol-rfcs/blob/message/text/0017-message/0017-message.md>
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// network_id specifies which network this message is meant for (mainnet, testnet, private net).
    network_id: u64,
    /// parents are the [MessageId]s that this message directly approves.
    /// The list contains between 1 and 8 sorted unique entries.
    parents: Parents,
    /// The optional [Payload] of the message.
    payload: Option<Payload>,
    /// The result of the Proof of Work (PoW) in order to be accepted into the tangle.
    nonce: u64,
}

impl Message {
    /// Create a new [MessageBuilder], the only way to construct an instance of a `Message`.
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Calculate and return the the ID of the message.
    ///
    /// The ID is obtained by taking the Blake2b256 hash of the message contents.
    ///
    /// TODO: should not return bytes anymore ?
    pub fn id(&self) -> (MessageId, Vec<u8>) {
        let bytes = self.pack_new();
        let id = Blake2b256::digest(&bytes);

        (MessageId::new(id.into()), bytes)
    }

    /// Return the ID of the network this message belongs to.
    ///
    /// This is used to indicate if the message is for the mainnet, testnet, or a private net.
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Return the set of transactions that this message directly approves (the parents).
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Return the (optional) payload this message contains.
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }

    /// Return the PoW nonce for this message.
    pub fn nonce(&self) -> u64 {
        self.nonce
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

/// A `MessageBuilder` is how you construct a [Message].
pub struct MessageBuilder<P: Provider = Miner> {
    network_id: Option<u64>,
    parents: Option<Parents>,
    payload: Option<Payload>,
    nonce_provider: Option<(P, f64, Option<Arc<AtomicBool>>)>,
}

impl<P: Provider> Default for MessageBuilder<P> {
    fn default() -> Self {
        Self {
            network_id: None,
            parents: None,
            payload: None,
            nonce_provider: None,
        }
    }
}

impl<P: Provider> MessageBuilder<P> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_network_id(mut self, network_id: u64) -> Self {
        self.network_id = Some(network_id);
        self
    }

    pub fn with_parents(mut self, parents: Parents) -> Self {
        self.parents = Some(parents);
        self
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_nonce_provider(mut self, nonce_provider: P, target_score: f64, done: Option<Arc<AtomicBool>>) -> Self {
        self.nonce_provider = Some((nonce_provider, target_score, done));
        self
    }

    pub fn finish(self) -> Result<Message, Error> {
        // TODO harmonize unpack and finish
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

        let (nonce_provider, target_score, done) =
            self.nonce_provider
                .unwrap_or((P::Builder::new().finish(), 4000f64, None));

        message.nonce = nonce_provider
            .nonce(
                &message_bytes[..message_bytes.len() - std::mem::size_of::<u64>()],
                target_score,
                done,
            )
            .unwrap_or(0);

        Ok(message)
    }
}
