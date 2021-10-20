// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::TokenId, Error};

use bee_common::packable::{Packable, Read, Write};

use primitive_types::U256;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct NativeToken {
    token_id: TokenId,
    amount: U256,
}

impl NativeToken {
    /// Creates a new `NativeToken`.
    pub fn new(token_id: TokenId, amount: U256) -> Self {
        Self { token_id, amount }
    }
}

impl Packable for NativeToken {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.token_id.packed_len() + 32
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.token_id.pack(writer)?;
        // SAFETY: Reinterpreting a [u64; 4] as a [u8; 32] is fine since they have the same size.
        writer.write_all(&unsafe { std::mem::transmute::<[u64; 4], [u8; 32]>(self.amount.0) })?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let token_id = TokenId::unpack_inner::<R, CHECK>(reader)?;
        let amount = U256::from_little_endian(&<[u8; 32]>::unpack_inner::<R, CHECK>(reader)?);

        Ok(Self::new(token_id, amount))
    }
}
