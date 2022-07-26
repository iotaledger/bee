// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use bee_pow::providers::{miner::Miner, NonceProvider, NonceProviderBuilder};
use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{
    error::{UnexpectedEOF, UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::{CounterUnpacker, SliceUnpacker, Unpacker},
    Packable, PackableExt,
};

use crate::{
    constant::PROTOCOL_VERSION,
    parent::Parents,
    payload::{OptionalPayload, Payload},
    BlockId, Error,
};

/// A builder to build a [`Block`].
#[derive(Clone)]
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

        let mut block = Block {
            protocol_version: PROTOCOL_VERSION,
            parents: self.parents,
            payload: self.payload.into(),
            nonce: 0,
        };

        let block_bytes = block.pack_to_vec();

        if block_bytes.len() > Block::LENGTH_MAX {
            return Err(Error::InvalidBlockLength(block_bytes.len()));
        }

        let (nonce_provider, target_score) = self
            .nonce_provider
            .unwrap_or((P::Builder::new().finish(), Self::DEFAULT_POW_SCORE));

        block.nonce = nonce_provider
            .nonce(
                &block_bytes[..block_bytes.len() - core::mem::size_of::<u64>()],
                target_score,
            )
            .unwrap_or(Self::DEFAULT_NONCE);

        Ok(block)
    }
}

/// Represent the object that nodes gossip around the network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    /// Protocol version of the block.
    protocol_version: u8,
    /// The [`BlockId`]s that this block directly approves.
    parents: Parents,
    /// The optional [Payload] of the block.
    payload: OptionalPayload,
    /// The result of the Proof of Work in order for the block to be accepted into the tangle.
    nonce: u64,
}

impl Block {
    /// The minimum number of bytes in a block.
    pub const LENGTH_MIN: usize = 46;
    /// The maximum number of bytes in a block.
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

    /// Computes the identifier of the block.
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
        visitor: &<Self as Packable>::UnpackVisitor,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>> {
        let mut unpacker = CounterUnpacker::new(SliceUnpacker::new(bytes.as_ref()));
        let block = Self::unpack::<_, true>(&mut unpacker, visitor)?;

        // When parsing the block is complete, there should not be any trailing bytes left that were not parsed.
        if u8::unpack::<_, true>(&mut unpacker, visitor).is_ok() {
            return Err(UnpackError::Packable(Error::RemainingBytesAfterBlock));
        }

        Ok(block)
    }
}

impl Packable for Block {
    type UnpackError = Error;
    type UnpackVisitor = ();

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.protocol_version.pack(packer)?;
        self.parents.pack(packer)?;
        self.payload.pack(packer)?;
        self.nonce.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let start_opt = unpacker.read_bytes();

        let protocol_version = u8::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;

        if VERIFY && protocol_version != PROTOCOL_VERSION {
            return Err(UnpackError::Packable(Error::ProtocolVersionMismatch {
                expected: PROTOCOL_VERSION,
                actual: protocol_version,
            }));
        }

        let parents = Parents::unpack::<_, VERIFY>(unpacker, visitor)?;
        let payload = OptionalPayload::unpack::<_, VERIFY>(unpacker, visitor)?;

        if VERIFY {
            verify_payload(payload.deref().as_ref()).map_err(UnpackError::Packable)?;
        }

        let nonce = u64::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;

        let block = Self {
            protocol_version,
            parents,
            payload,
            nonce,
        };

        if VERIFY {
            let block_len = if let (Some(start), Some(end)) = (start_opt, unpacker.read_bytes()) {
                end - start
            } else {
                block.packed_len()
            };

            if block_len > Block::LENGTH_MAX {
                return Err(UnpackError::Packable(Error::InvalidBlockLength(block_len)));
            }
        }

        Ok(block)
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

    /// The block object that nodes gossip around in the network.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct BlockDto {
        ///
        #[serde(rename = "protocolVersion")]
        pub protocol_version: u8,
        ///
        pub parents: Vec<String>,
        ///
        #[serde(skip_serializing_if = "Option::is_none")]
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
                    .map(|m| m.parse::<BlockId>().map_err(|_| DtoError::InvalidField("parents")))
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

#[cfg(feature = "inx")]
mod inx {
    use super::*;

    impl TryFrom<inx_bindings::proto::RawBlock> for Block {
        type Error = crate::error::inx::InxError;

        fn try_from(value: inx_bindings::proto::RawBlock) -> Result<Self, Self::Error> {
            Self::unpack_verified(value.data, &mut ()).map_err(|e| Self::Error::InvalidRawBytes(e.to_string()))
        }
    }
}
