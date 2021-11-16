// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, Error};

use bee_common::packable::{Packable, Read, Write};

/// Identifies the validated issuer of the output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct IssuerFeatureBlock(Address);

impl IssuerFeatureBlock {
    /// The feature block kind of an `IssuerFeatureBlock`.
    pub const KIND: u8 = 1;

    /// Creates a new `IssuerFeatureBlock`.
    pub fn new(address: Address) -> Self {
        address.into()
    }

    /// Returns the issuer address.
    pub fn address(&self) -> &Address {
        &self.0
    }
}

impl Packable for IssuerFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(Address::unpack_inner::<R, CHECK>(reader)?))
    }
}
