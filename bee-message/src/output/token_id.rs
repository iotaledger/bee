// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::FoundryId;

impl_id!(pub TokenId, 38, "TODO.");

#[cfg(feature = "serde")]
string_serde_impl!(TokenId);

impl From<FoundryId> for TokenId {
    fn from(foundry_id: FoundryId) -> Self {
        TokenId::new(*foundry_id)
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    /// Describes a token id.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct TokenIdDto(pub String);

    impl From<&TokenId> for TokenIdDto {
        fn from(value: &TokenId) -> Self {
            Self(prefix_hex::encode(**value))
        }
    }

    impl TryFrom<&TokenIdDto> for TokenId {
        type Error = DtoError;

        fn try_from(value: &TokenIdDto) -> Result<Self, Self::Error> {
            let token_id: [u8; TokenId::LENGTH] =
                prefix_hex::decode(&value.0).map_err(|_e| DtoError::InvalidField("tokenId"))?;
            Ok(TokenId::new(token_id))
        }
    }
}
