// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::OutputId;

impl_id!(pub NftId, 32, "TODO.");

#[cfg(feature = "serde")]
string_serde_impl!(NftId);

impl From<OutputId> for NftId {
    fn from(output_id: OutputId) -> Self {
        Self::from(output_id.hash())
    }
}

impl NftId {
    ///
    pub fn or_from_output_id(self, output_id: OutputId) -> Self {
        if self.is_null() { Self::from(output_id) } else { self }
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct NftIdDto(pub String);

    impl From<&NftId> for NftIdDto {
        fn from(value: &NftId) -> Self {
            Self(value.to_string())
        }
    }

    impl TryFrom<&NftIdDto> for NftId {
        type Error = DtoError;

        fn try_from(value: &NftIdDto) -> Result<Self, Self::Error> {
            value.0.parse::<NftId>().map_err(|_| DtoError::InvalidField("NFT id"))
        }
    }
}
