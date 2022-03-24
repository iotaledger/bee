// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

///
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidTokenSchemeKind)]
pub enum TokenScheme {
    ///
    Simple = 0,
}

impl TryFrom<u8> for TokenScheme {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TokenScheme::Simple),
            k => Err(Error::InvalidTokenSchemeKind(k)),
        }
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TokenSchemeDto {
        #[serde(rename = "type")]
        pub kind: u8,
    }
}
