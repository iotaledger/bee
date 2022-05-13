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
    BlockId, Error,
};

/// A builder to build a [`Block`].
#[must_use]
pub struct BlockBuilder<P: NonceProvider = Miner> {
    parents: Parents,
    payload: Option<Payload>,
    nonce_provider: Option<(P, f64)>,
}

impl<P: NonceProvider> BlockBuilder<P> {
    const DEFAULT_POW_SCORE: f64 = 4000f64;
    const DEFAULT_NONCE: u64 = 0;

    /// Creates a new [`BlockBuilder`].
    #[inline(always)]
    pub fn new(parents: Parents) -> Self {
        Self {
            parents,
            payload: None,
            nonce_provider: None,
        }
    }

    /// Adds a payload to a [`BlockBuilder`].
    #[inline(always)]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Adds a nonce provider to a [`BlockBuilder`].
    #[inline(always)]
    pub fn with_nonce_provider(mut self, nonce_provider: P, target_score: f64) -> Self {
        self.nonce_provider = Some((nonce_provider, target_score));
        self
    }

    /// Finishes the [`BlockBuilder`] into a [`Block`].
    pub fn finish(self) -> Result<Block, Error> {
        verify_payload(self.payload.as_ref())?;

        let mut message = Block {
            protocol_version: PROTOCOL_VERSION,
            parents: self.parents,
            payload: self.payload.into(),
            nonce: 0,
        };

        let message_bytes = message.pack_to_vec();

        if message_bytes.len() > Block::LENGTH_MAX {
            return Err(Error::InvalidBlockLength(message_bytes.len()));
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
pub struct Block {
    /// Protocol version of the message.
    protocol_version: u8,
    /// The [`BlockId`]s that this message directly approves.
    parents: Parents,
    /// The optional [Payload] of the message.
    payload: OptionalPayload,
    /// The result of the Proof of Work in order for the message to be accepted into the tangle.
    nonce: u64,
}

impl Block {
    /// The minimum number of bytes in a message.
    pub const LENGTH_MIN: usize = 46;
    /// The maximum number of bytes in a message.
    pub const LENGTH_MAX: usize = 32768;

    /// Creates a new [`BlockBuilder`] to construct an instance of a [`Block`].
    #[inline(always)]
    pub fn build(parents: Parents) -> BlockBuilder {
        BlockBuilder::new(parents)
    }

    /// Returns the protocol version of a [`Block`].
    #[inline(always)]
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version
    }

    /// Returns the parents of a [`Block`].
    #[inline(always)]
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the optional payload of a [`Block`].
    #[inline(always)]
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    /// Returns the nonce of a [`Block`].
    #[inline(always)]
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Computes the identifier of the message.
    #[inline(always)]
    pub fn id(&self) -> BlockId {
        BlockId::new(Blake2b256::digest(&self.pack_to_vec()).into())
    }

    /// Consumes the [`Block`], and returns ownership over its [`Parents`].
    #[inline(always)]
    pub fn into_parents(self) -> Parents {
        self.parents
    }

    /// Unpacks a [`Block`] from a sequence of bytes doing syntactical checks and verifying that
    /// there are no trailing bytes in the sequence.
    pub fn unpack_strict<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>> {
        let mut unpacker = CounterUnpacker::new(bytes.as_ref());
        let message = Self::unpack::<_, true>(&mut unpacker)?;

        // When parsing the message is complete, there should not be any trailing bytes left that were not parsed.
        if u8::unpack::<_, true>(&mut unpacker).is_ok() {
            return Err(UnpackError::Packable(Error::RemainingBytesAfterBlock));
        }

        Ok(message)
    }
}

impl Packable for Block {
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

        if VERIFY {
            let message_len = if let (Some(start), Some(end)) = (start_opt, unpacker.read_bytes()) {
                end - start
            } else {
                message.packed_len()
            };

            if message_len > Block::LENGTH_MAX {
                return Err(UnpackError::Packable(Error::InvalidBlockLength(message_len)));
            }
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
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct BlockDto {
        ///
        #[serde(rename = "protocolVersion")]
        pub protocol_version: u8,
        #[serde(rename = "parentBlockIds")]
        ///
        pub parents: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ///
        pub payload: Option<PayloadDto>,
        ///
        pub nonce: String,
    }

    impl From<&Block> for BlockDto {
        fn from(value: &Block) -> Self {
            BlockDto {
                protocol_version: value.protocol_version(),
                parents: value.parents().iter().map(BlockId::to_string).collect(),
                payload: value.payload().map(Into::into),
                nonce: value.nonce().to_string(),
            }
        }
    }

    impl TryFrom<&BlockDto> for Block {
        type Error = DtoError;

        fn try_from(value: &BlockDto) -> Result<Self, Self::Error> {
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
                        m.parse::<BlockId>()
                            .map_err(|_| DtoError::InvalidField("parentBlockIds"))
                    })
                    .collect::<Result<Vec<BlockId>, DtoError>>()?,
            )?;

            let mut builder = BlockBuilder::new(parents).with_nonce_provider(
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
