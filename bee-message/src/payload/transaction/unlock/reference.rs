// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::transaction::constants::INPUT_OUTPUT_INDEX_RANGE, Error};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use core::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ReferenceUnlock(u16);

impl TryFrom<u16> for ReferenceUnlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Self::Error::InvalidIndex);
        }

        Ok(Self(index))
    }
}

impl ReferenceUnlock {
    pub fn new(index: u16) -> Result<Self, Error> {
        index.try_into()
    }

    pub fn index(&self) -> u16 {
        self.0
    }
}

impl Packable for ReferenceUnlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::new(u16::unpack(reader)?).map_err(|_| Self::Error::InvalidSyntax)?)
    }
}
