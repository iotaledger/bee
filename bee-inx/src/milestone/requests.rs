// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Bound, RangeBounds};

use crate::{bee, error, inx};

/// Allows to request a milestone by either its index or its id.
#[allow(missing_docs)]
pub enum MilestoneRequest {
    MilestoneIndex(bee::MilestoneIndex),
    MilestoneId(bee::MilestoneId),
}

impl From<MilestoneRequest> for inx::MilestoneRequest {
    fn from(value: MilestoneRequest) -> Self {
        match value {
            MilestoneRequest::MilestoneIndex(bee::MilestoneIndex(milestone_index)) => Self {
                milestone_index,
                milestone_id: None,
            },
            MilestoneRequest::MilestoneId(milestone_id) => Self {
                milestone_index: 0,
                milestone_id: Some(milestone_id.into()),
            },
        }
    }
}

impl<T: Into<u32>> From<T> for MilestoneRequest {
    fn from(value: T) -> Self {
        Self::MilestoneIndex(bee::MilestoneIndex(value.into()))
    }
}

/// A request for a range of milestones by [`bee::MilestoneIndex`].
#[derive(Clone, Debug, PartialEq)]
pub struct MilestoneRangeRequest(inx::MilestoneRangeRequest);

impl<T> From<T> for MilestoneRangeRequest
where
    T: RangeBounds<u32>,
{
    fn from(value: T) -> MilestoneRangeRequest {
        MilestoneRangeRequest(to_milestone_range_request(value))
    }
}

impl From<MilestoneRangeRequest> for inx::MilestoneRangeRequest {
    fn from(value: MilestoneRangeRequest) -> Self {
        value.0
    }
}

fn to_milestone_range_request<T, I>(range: T) -> inx::MilestoneRangeRequest
where
    T: RangeBounds<I>,
    I: Into<u32> + Copy,
{
    let start_milestone_index = match range.start_bound() {
        Bound::Included(&idx) => idx.into(),
        Bound::Excluded(&idx) => idx.into() + 1,
        Bound::Unbounded => 0,
    };
    let end_milestone_index = match range.end_bound() {
        Bound::Included(&idx) => idx.into(),
        Bound::Excluded(&idx) => idx.into() - 1,
        Bound::Unbounded => 0,
    };
    inx::MilestoneRangeRequest {
        start_milestone_index,
        end_milestone_index,
    }
}

/// Allows to request "white flag" data for a particular milestone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhiteFlagRequest {
    milestone_index: bee::MilestoneIndex,
    milestone_timestamp: u32,
    parents: Box<[bee::BlockId]>,
    previous_milestone_id: Option<bee::MilestoneId>,
}

impl TryFrom<inx::WhiteFlagRequest> for WhiteFlagRequest {
    type Error = error::Error;

    fn try_from(value: inx::WhiteFlagRequest) -> Result<Self, Self::Error> {
        let parents = value
            .parents
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            milestone_index: value.milestone_index.into(),
            milestone_timestamp: value.milestone_timestamp,
            parents: parents.into_boxed_slice(),
            previous_milestone_id: value.previous_milestone_id.map(TryInto::try_into).transpose()?,
        })
    }
}

impl From<WhiteFlagRequest> for inx::WhiteFlagRequest {
    fn from(value: WhiteFlagRequest) -> Self {
        Self {
            milestone_index: value.milestone_index.0,
            milestone_timestamp: value.milestone_timestamp,
            parents: value.parents.into_vec().into_iter().map(Into::into).collect(),
            previous_milestone_id: value.previous_milestone_id.map(Into::into),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn exclusive() {
        let range = MilestoneRangeRequest::from(17..43);
        assert_eq!(
            range,
            MilestoneRangeRequest(inx::MilestoneRangeRequest {
                start_milestone_index: 17,
                end_milestone_index: 42
            })
        );
    }

    #[test]
    fn inclusive() {
        let range = MilestoneRangeRequest::from(17..=42);
        assert_eq!(
            range,
            MilestoneRangeRequest(inx::MilestoneRangeRequest {
                start_milestone_index: 17,
                end_milestone_index: 42
            })
        );
    }
}
