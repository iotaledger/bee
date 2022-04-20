// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use bee_pow::providers::{miner::Miner, NonceProvider, NonceProviderBuilder};
use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{
    error::{UnexpectedEOF, UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::{CounterUnpacker, Unpacker},
    Packable, PackableExt,
};

use crate::{
    constant::PROTOCOL_VERSION,
    parent::Parents,
    payload::{OptionalPayload, Payload},
    Error, MessageId,
};

/// A builder to build a [`Message`].
#[must_use]
pub struct MessageBuilder<P: NonceProvider = Miner> {
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
            parents,
            payload: None,
            nonce_provider: None,
        }
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
        verify_payload(self.payload.as_ref())?;

        let mut message = Message {
            protocol_version: PROTOCOL_VERSION,
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn build(parents: Parents) -> MessageBuilder {
        MessageBuilder::new(parents)
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

    /// Computes the identifier of the message.
    #[inline(always)]
    pub fn id(&self) -> MessageId {
        MessageId::new(Blake2b256::digest(&self.pack_to_vec()).into())
    }

    /// Consumes the [`Message`], and returns ownership over its [`Parents`].
    #[inline(always)]
    pub fn into_parents(self) -> Parents {
        self.parents
    }

    /// Unpacks a [`Message`] from a sequence of bytes doing syntactical checks and verifying that
    /// there are no traling bytes in the secuence.
    pub fn unpack_strict<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>> {
        let mut unpacker = CounterUnpacker::new(bytes.as_ref());
        let message = Self::unpack::<_, true>(&mut unpacker)?;

        // When parsing the message is complete, there should not be any trailing bytes left that were not parsed.
        if u8::unpack::<_, true>(&mut unpacker).is_ok() {
            return Err(UnpackError::Packable(Error::RemainingBytesAfterMessage));
        }

        Ok(message)
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
        let start_opt = unpacker.read_bytes();

        let protocol_version = u8::unpack::<_, VERIFY>(unpacker).coerce()?;

        if VERIFY && protocol_version != PROTOCOL_VERSION {
            return Err(UnpackError::Packable(Error::ProtocolVersionMismatch {
                expected: PROTOCOL_VERSION,
                actual: protocol_version,
            }));
        }

        let parents = Parents::unpack::<_, VERIFY>(unpacker)?;
        let payload = OptionalPayload::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_payload(payload.deref().as_ref()).map_err(UnpackError::Packable)?;
        }

        let nonce = u64::unpack::<_, VERIFY>(unpacker).coerce()?;

        let message = Self {
            protocol_version,
            parents,
            payload,
            nonce,
        };

        let message_len = if let (Some(start), Some(end)) = (start_opt, unpacker.read_bytes()) {
            end - start
        } else {
            message.packed_len()
        };

        if VERIFY && message_len > Message::LENGTH_MAX {
            return Err(UnpackError::Packable(Error::InvalidMessageLength(message_len)));
        }

        Ok(message)
    }
}

fn verify_payload(payload: Option<&Payload>) -> Result<(), Error> {
    if !matches!(
        payload,
        None | Some(Payload::Transaction(_)) | Some(Payload::Milestone(_)) | Some(Payload::TaggedData(_))
    ) {
        // Safe to unwrap since it's known not to be None.
        Err(Error::InvalidPayloadKind(payload.unwrap().kind()))
    } else {
        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{error::dto::DtoError, payload::dto::PayloadDto};

    /// The message object that nodes gossip around in the network.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct MessageDto {
        ///
        #[serde(rename = "protocolVersion")]
        pub protocol_version: u8,
        #[serde(rename = "parentMessageIds")]
        ///
        pub parents: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ///
        pub payload: Option<PayloadDto>,
        ///
        pub nonce: String,
    }

    impl From<&Message> for MessageDto {
        fn from(value: &Message) -> Self {
            MessageDto {
                protocol_version: value.protocol_version(),
                parents: value.parents().iter().map(MessageId::to_string).collect(),
                payload: value.payload().map(Into::into),
                nonce: value.nonce().to_string(),
            }
        }
    }

    impl TryFrom<&MessageDto> for Message {
        type Error = DtoError;

        fn try_from(value: &MessageDto) -> Result<Self, Self::Error> {
            if value.protocol_version != PROTOCOL_VERSION {
                return Err(Error::ProtocolVersionMismatch {
                    expected: PROTOCOL_VERSION,
                    actual: value.protocol_version,
                }
                .into());
            }

            let parents = Parents::new(
                value
                    .parents
                    .iter()
                    .map(|m| {
                        m.parse::<MessageId>()
                            .map_err(|_| DtoError::InvalidField("parentMessageIds"))
                    })
                    .collect::<Result<Vec<MessageId>, DtoError>>()?,
            )?;

            let mut builder = MessageBuilder::new(parents).with_nonce_provider(
                value
                    .nonce
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("nonce"))?,
                0f64,
            );
            if let Some(p) = value.payload.as_ref() {
                builder = builder.with_payload(p.try_into()?);
            }

            Ok(builder.finish()?)
        }
    }
}
