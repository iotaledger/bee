// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::DUST_DEPOSIT_MIN, Error};

use bee_common::packable::{Packable, Read, Write};

/// Defines the amount of IOTAs used as dust deposit that have to be returned to the sender [`Address`].
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct DustDepositReturnFeatureBlock {
    // Amount of IOTA coins the consuming transaction should deposit to the [`Address`] defined in
    // [`SenderFeatureBlock`].
    amount: u64,
}

impl TryFrom<u64> for DustDepositReturnFeatureBlock {
    type Error = Error;

    fn try_from(amount: u64) -> Result<Self, Self::Error> {
        validate_amount(amount)?;

        Ok(Self { amount })
    }
}

impl DustDepositReturnFeatureBlock {
    /// The [`FeatureBlock`] kind of a [`DustDepositReturnFeatureBlock`].
    pub const KIND: u8 = 2;

    /// Creates a new [`DustDepositReturnFeatureBlock`].
    pub fn new(amount: u64) -> Result<Self, Error> {
        Self::try_from(amount)
    }

    /// Returns the amount.
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for DustDepositReturnFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_amount(amount)?;
        }

        Ok(Self { amount })
    }
}

#[inline]
fn validate_amount(amount: u64) -> Result<(), Error> {
    if amount < DUST_DEPOSIT_MIN {
        return Err(Error::InvalidDustDepositAmount(amount));
    }

    Ok(())
}
