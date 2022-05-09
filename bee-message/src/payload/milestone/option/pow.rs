// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the PoW milestone option.

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use crate::Error;

///
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PowMilestoneOption {
    next_pow_score: u32,
    next_pow_score_milestone_index: u32,
}

impl PowMilestoneOption {
    /// The milestone option kind of a [`PowMilestoneOption`].
    pub const KIND: u8 = 1;

    /// Creates a new [`PowMilestoneOption`].
    pub fn new(next_pow_score: u32, next_pow_score_milestone_index: u32) -> Result<Self, Error> {
        // TODO put back when the TIP is finished
        // verify_pow_scores(index, next_pow_score, next_pow_score_milestone_index)?;

        Ok(Self {
            next_pow_score,
            next_pow_score_milestone_index,
        })
    }

    /// Returns the next proof of work score of a [`PowMilestoneOption`].
    pub fn next_pow_score(&self) -> u32 {
        self.next_pow_score
    }

    /// Returns the next proof of work index of a [`PowMilestoneOption`].
    pub fn next_pow_score_milestone_index(&self) -> u32 {
        self.next_pow_score_milestone_index
    }
}

impl Packable for PowMilestoneOption {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.next_pow_score.pack(packer)?;
        self.next_pow_score_milestone_index.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let next_pow_score = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let next_pow_score_milestone_index = u32::unpack::<_, VERIFY>(unpacker).coerce()?;

        // TODO put back when the TIP is finished
        // if VERIFY {
        //     verify_pow_scores(index, next_pow_score, next_pow_score_milestone_index).map_err(UnpackError::Packable)?;
        // }

        Ok(Self {
            next_pow_score,
            next_pow_score_milestone_index,
        })
    }
}

// TODO put back when the TIP is finished
// fn verify_pow_scores(
//     index: MilestoneIndex,
//     next_pow_score: u32,
//     next_pow_score_milestone_index: u32,
// ) -> Result<(), Error> {
//     if next_pow_score == 0 && next_pow_score_milestone_index != 0
//         || next_pow_score != 0 && next_pow_score_milestone_index <= *index
//     {
//         Err(Error::InvalidPowScoreValues {
//             nps: next_pow_score,
//             npsmi: next_pow_score_milestone_index,
//         })
//     } else {
//         Ok(())
//     }
// }

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    ///
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct PowMilestoneOptionDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "nextPoWScore")]
        pub next_pow_score: u32,
        #[serde(rename = "nextPoWScoreMilestoneIndex")]
        pub next_pow_score_milestone_index: u32,
    }

    impl From<&PowMilestoneOption> for PowMilestoneOptionDto {
        fn from(value: &PowMilestoneOption) -> Self {
            PowMilestoneOptionDto {
                kind: PowMilestoneOption::KIND,
                next_pow_score: value.next_pow_score(),
                next_pow_score_milestone_index: value.next_pow_score_milestone_index(),
            }
        }
    }

    impl TryFrom<&PowMilestoneOptionDto> for PowMilestoneOption {
        type Error = DtoError;

        fn try_from(value: &PowMilestoneOptionDto) -> Result<Self, Self::Error> {
            Ok(PowMilestoneOption::new(
                value.next_pow_score,
                value.next_pow_score_milestone_index,
            )?)
        }
    }
}
