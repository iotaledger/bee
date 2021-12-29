// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parent::Parents,
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, OptionalPayload, Payload},
    Error, MessageId,
};

use bee_common::packable::{Read, Write};
use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
};
use bee_pow::providers::{miner::Miner, NonceProvider, NonceProviderBuilder};

use crypto::hashes::{blake2b::Blake2b256, Digest};

/// A builder to build a [`Message`].
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
    const DEFAULT_POW_SCORE: f64 = 4000f64;
    const DEFAULT_NONCE: u64 = 0;

    /// Creates a new `MessageBuilder`.
    #[inline(always)]
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a network id to a `MessageBuilder`.
    #[inline(always)]
    pub fn with_network_id(mut self, network_id: u64) -> Self {
        self.network_id = Some(network_id);
        self
    }

    /// Adds parents to a `MessageBuilder`.
    #[inline(always)]
    pub fn with_parents(mut self, parents: Parents) -> Self {
        self.parents = Some(parents);
        self
    }

    /// Adds a payload to a `MessageBuilder`.
    #[inline(always)]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Adds a nonce provider to a `MessageBuilder`.
    #[inline(always)]
    pub fn with_nonce_provider(mut self, nonce_provider: P, target_score: f64) -> Self {
        self.nonce_provider = Some((nonce_provider, target_score));
        self
    }

    /// Finishes the `MessageBuilder` into a [`Message`].
    pub fn finish(self) -> Result<Message, Error> {
        use bee_common::packable::Packable;

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
            payload: self.payload.into(),
            nonce: 0,
        };

        let message_bytes = message.pack_new();

        if message_bytes.len() > Message::LENGTH_MAX {
            return Err(Error::InvalidMessageLength(message_bytes.len()));
        }

        let (nonce_provider, target_score) = self
            .nonce_provider
            .unwrap_or((P::Builder::new().finish(), Self::DEFAULT_POW_SCORE));

        message.nonce = nonce_provider
            .nonce(
                &message_bytes[..message_bytes.len() - std::mem::size_of::<u64>()],
                target_score,
            )
            .unwrap_or(Self::DEFAULT_NONCE);

        Ok(message)
    }
}

/// Represent the object that nodes gossip around the network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// Specifies which network this message is meant for.
    network_id: u64,
    /// The [`MessageId`]s that this message directly approves.
    parents: Parents,
    /// The optional [Payload] of the message.
    payload: OptionalPayload,
    /// The result of the Proof of Work in order for the message to be accepted into the tangle.
    nonce: u64,
}

impl Message {
    /// The minimum number of bytes in a message.
    pub const LENGTH_MIN: usize = 53;
    /// The maximum number of bytes in a message.
    pub const LENGTH_MAX: usize = 32768;

    /// Creates a new `MessageBuilder` to construct an instance of a [`Message`].
    #[inline(always)]
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Computes the identifier of the message.
    #[inline(always)]
    pub fn id(&self) -> MessageId {
        use bee_common::packable::Packable;
        MessageId::new(Blake2b256::digest(&self.pack_new()).into())
    }

    /// Returns the network id of a [`Message`].
    #[inline(always)]
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Returns the parents of a [`Message`].
    #[inline(always)]
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the optional payload of a [`Message`].
    #[inline(always)]
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    /// Returns the nonce of a [`Message`].
    #[inline(always)]
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Consumes the [[`Message`]], and returns ownership over its [`Parents`].
    #[inline(always)]
    pub fn into_parents(self) -> Parents {
        self.parents
    }

    /// Consumes the [[`Message`]], and returns ownership over its [`Parents`] and [`Payload`].
    pub fn into_parents_and_payload(self) -> (Parents, Option<Payload>) {
        (self.parents, self.payload.into())
    }
}

impl bee_packable::Packable for Message {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.network_id.pack(packer)?;
        self.parents.pack(packer)?;
        self.payload.pack(packer)?;
        self.nonce.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        use bee_packable::PackableExt;

        let network_id = u64::unpack::<_, VERIFY>(unpacker).infallible()?;

        let parents = Parents::unpack::<_, VERIFY>(unpacker)?;

        let payload = OptionalPayload::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY
            && !matches!(
                *payload,
                None | Some(Payload::Transaction(_)) | Some(Payload::Milestone(_)) | Some(Payload::Indexation(_))
            )
        {
            // Safe to unwrap since it's known not to be None.
            return Err(UnpackError::Packable(Error::InvalidPayloadKind(
                Into::<Option<Payload>>::into(payload).unwrap().kind(),
            )));
        }

        let nonce = u64::unpack::<_, VERIFY>(unpacker).infallible()?;

        let message = Self {
            network_id,
            parents,
            payload,
            nonce,
        };

        let message_len = message.packed_len();

        // FIXME: compute this in a more efficient way.
        if VERIFY && message_len > Message::LENGTH_MAX {
            return Err(UnpackError::Packable(Error::InvalidMessageLength(message_len)));
        }

        // When parsing the message is complete, there should not be any trailing bytes left that were not parsed.
        if VERIFY && u8::unpack::<_, VERIFY>(unpacker).is_ok() {
            return Err(UnpackError::Packable(Error::RemainingBytesAfterMessage));
        }

        Ok(message)
    }
}

impl bee_common::packable::Packable for Message {
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

        if CHECK && message_len > Message::LENGTH_MAX {
            return Err(Error::InvalidMessageLength(message_len));
        }

        // When parsing the message is complete, there should not be any trailing bytes left that were not parsed.
        if CHECK && reader.bytes().next().is_some() {
            return Err(Error::RemainingBytesAfterMessage);
        }

        Ok(Self {
            network_id,
            parents,
            payload: payload.into(),
            nonce,
        })
    }
}
