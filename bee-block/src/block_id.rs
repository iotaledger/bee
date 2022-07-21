// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

impl_id!(
    pub BlockId,
    32,
    "A block identifier, the BLAKE2b-256 hash of the block bytes. See <https://www.blake2.net/> for more information."
);

#[cfg(feature = "serde")]
string_serde_impl!(BlockId);

#[cfg(feature = "inx")]
mod inx {
    use super::*;

    impl From<BlockId> for inx_bindings::proto::BlockId {
        fn from(value: BlockId) -> Self {
            Self { id: value.0.to_vec() }
        }
    }

    impl TryFrom<inx_bindings::proto::BlockId> for BlockId {
        type Error = crate::error::inx::InxError;

        fn try_from(value: inx_bindings::proto::BlockId) -> Result<Self, Self::Error> {
            let bytes: [u8; BlockId::LENGTH] = value.id.try_into().map_err(|e| Self::Error::InvalidId("BlockId", e))?;
            Ok(BlockId::from(bytes))
        }
    }
}
