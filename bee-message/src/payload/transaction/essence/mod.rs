// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod regular;

pub use regular::{RegularEssence, RegularEssenceBuilder};

use regular::REGULAR_ESSENCE_KIND;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Essence {
    Regular(RegularEssence),
}

impl From<RegularEssence> for Essence {
    fn from(essence: RegularEssence) -> Self {
        Self::Regular(essence)
    }
}

impl Essence {
    pub fn kind(&self) -> u8 {
        match self {
            Self::Regular(_) => REGULAR_ESSENCE_KIND,
        }
    }
}

impl Packable for Essence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Regular(essence) => REGULAR_ESSENCE_KIND.packed_len() + essence.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Regular(essence) => {
                REGULAR_ESSENCE_KIND.pack(writer)?;
                essence.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            REGULAR_ESSENCE_KIND => Self::Regular(RegularEssence::unpack(reader)?),
            k => return Err(Self::Error::InvalidEssenceKind(k)),
        })
    }
}
