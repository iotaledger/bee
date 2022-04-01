// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::milestone::MilestoneIndex;

/// Describes a U256.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct U256Dto(pub String);

impl From<&U256> for U256Dto {
    fn from(value: &U256) -> Self {
        Self(prefix_hex::encode(*value))
    }
}

impl TryFrom<&U256Dto> for U256 {
    type Error = prefix_hex::Error;

    fn try_from(value: &U256Dto) -> Result<Self, Self::Error> {
        prefix_hex::decode(&value.0)
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_zero(num: &u32) -> bool {
    *num == 0
}

pub fn is_zero_milestone(num: &MilestoneIndex) -> bool {
    num.0 == 0
}
