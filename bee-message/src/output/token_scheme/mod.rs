// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod simple;

pub use simple::SimpleTokenScheme;

use crate::Error;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidTokenSchemeKind)]
pub enum TokenScheme {
    ///
    #[packable(tag = SimpleTokenScheme::KIND)]
    Simple(SimpleTokenScheme),
}

impl TokenScheme {
    /// Returns the token scheme kind of a [`TokenScheme`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Simple(_) => SimpleTokenScheme::KIND,
        }
    }
}
