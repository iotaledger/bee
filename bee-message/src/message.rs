// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parent::Parents,
    payload::{OptionalPayload, Payload},
    Error, MessageId,
};

use bee_pow::providers::{miner::Miner, NonceProvider, NonceProviderBuilder};

use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

/// A builder to build a [`Message`].
#[must_use]
pub struct MessageBuilder<P: NonceProvider = Miner> {
    protocol_version: Option<u8>,
    parents: Parents,
    payload: Option<Payload>,
    nonce_provider: Option<(P, f64)>,
}

impl<P: NonceProvider> MessageBuilder<P> {
    const DEFAULT_POW_SCORE: f64 = 4000f64;
    const DEFAULT_NONCE: u64 = 0;

    /// Creates a new [`MessageBuilder`].
    #[inline(always)]
    pub fn new(parents: Parents) -> Self {
        Self {
            protocol_version: None,
            parents,
            payload: None,
            nonce_provider: None,
        }
    }

    /// Adds a protocol version to a [`MessageBuilder`].
    #[inline(always)]
    pub fn with_protocol_version(mut self, protocol_version: u8) -> Self {
        self.protocol_version = Some(protocol_version);
        self
    }

    /// Adds a payload to a [`MessageBuilder`].
    #[inline(always)]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Adds a nonce provider to a [`MessageBuilder`].
    #[inline(always)]
    pub fn with_nonce_provider(mut self, nonce_provider: P, target_score: f64) -> Self {
        self.nonce_provider = Some((nonce_provider, target_score));
        self
    }

    /// Finishes the [`MessageBuilder`] into a [`Message`].
    pub fn finish(self) -> Result<Message, Error> {
        let protocol_version = self.protocol_version.ok_or(Error::MissingField("protocol_version"))?;

        if !matches!(
            self.payload,
            None | Some(Payload::Transaction(_)) | Some(Payload::Milestone(_)) | Some(Payload::TaggedData(_))
        ) {
            // Safe to unwrap since it's known not to be None.
            return Err(Error::InvalidPayloadKind(self.payload.unwrap().kind()));
        }

        let mut message = Message {
            protocol_version,
            parents: self.parents,
            payload: self.payload.into(),
            nonce: 0,
        };

        let message_bytes = message.pack_to_vec();

        if message_bytes.len() > Message::LENGTH_MAX {
            return Err(Error::InvalidMessageLength(message_bytes.len()));
        }

        let (nonce_provider, target_score) = self
            .nonce_provider
            .unwrap_or((P::Builder::new().finish(), Self::DEFAULT_POW_SCORE));

        message.nonce = nonce_provider
            .nonce(
                &message_bytes[..message_bytes.len() - core::mem::size_of::<u64>()],
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
    /// Protocol version of the message.
    protocol_version: u8,
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

    /// Creates a new [`MessageBuilder`] to construct an instance of a [`Message`].
    #[inline(always)]
    pub fn builder(parents: Parents) -> MessageBuilder {
        MessageBuilder::new(parents)
    }

    /// Computes the identifier of the message.
    #[inline(always)]
    pub fn id(&self) -> MessageId {
        MessageId::new(Blake2b256::digest(&self.pack_to_vec()).into())
    }

    /// Returns the protocol version of a [`Message`].
    #[inline(always)]
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version
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

    /// Consumes the [`Message`], and returns ownership over its [`Parents`].
    #[inline(always)]
    pub fn into_parents(self) -> Parents {
        self.parents
    }
}

impl Packable for Message {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.protocol_version.pack(packer)?;
        self.parents.pack(packer)?;
        self.payload.pack(packer)?;
        self.nonce.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let protocol_version = u8::unpack::<_, VERIFY>(unpacker).infallible()?;

        let parents = Parents::unpack::<_, VERIFY>(unpacker)?;

        let payload = OptionalPayload::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY
            && !matches!(
                *payload,
                None | Some(Payload::Transaction(_)) | Some(Payload::Milestone(_)) | Some(Payload::TaggedData(_))
            )
        {
            // Safe to unwrap since it's known not to be None.
            return Err(UnpackError::Packable(Error::InvalidPayloadKind(
                Into::<Option<Payload>>::into(payload).unwrap().kind(),
            )));
        }

        let nonce = u64::unpack::<_, VERIFY>(unpacker).infallible()?;

        let message = Self {
            protocol_version,
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
