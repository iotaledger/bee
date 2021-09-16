// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageUnpackError;

use bee_packable::Packable;

use core::{convert::Infallible, fmt};

/// Error encountered when unpacking an [`Opinion`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum OpinionUnpackError {
    InvalidKind(u8),
}

impl fmt::Display for OpinionUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKind(kind) => write!(f, "invalid Opinion kind: {}", kind),
        }
    }
}

impl_from_infallible!(OpinionUnpackError);

/// Defines an opinion.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(tag_type = u8, with_error = OpinionUnpackError::InvalidKind)]
#[packable(unpack_error = MessageUnpackError)]
pub enum Opinion {
    /// Defines a "liked" opinion.
    #[packable(tag = 1)]
    Like,
    /// Defines a "disliked" opinion.
    #[packable(tag = 2)]
    Dislike,
    /// Defines an "unknown" opinion.
    #[packable(tag = 4)]
    Unknown,
}

impl Default for Opinion {
    fn default() -> Self {
        Opinion::Unknown
    }
}

impl fmt::Display for Opinion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
