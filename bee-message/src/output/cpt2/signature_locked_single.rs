// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, output::OutputAmount, Error};
use packable::Packable;

/// Describes an extended output with optional features.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct SignatureLockedSingleOutput {
    address: Address,
    // Amount of IOTA tokens held by the output.
    #[packable(unpack_error_with = Error::InvalidOutputAmount)]
    amount: OutputAmount,
}

impl SignatureLockedSingleOutput {
    /// The [`Output`](crate::output::Output) kind of an [`SignatureLockedSingleOutput`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SignatureLockedSingleOutput`].
    #[inline(always)]
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        Ok(Self {
            address,
            amount: amount.try_into().map_err(Error::InvalidOutputAmount)?,
        })
    }

    ///
    #[inline(always)]
    pub fn address(&self) -> &Address {
        &self.address
    }

    ///
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
