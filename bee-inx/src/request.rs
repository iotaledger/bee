// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Bound, RangeBounds};

use bee_block::payload::milestone::{MilestoneId, MilestoneIndex};
use inx::proto;

pub enum MilestoneRequest {
    MilestoneIndex(MilestoneIndex),
    MilestoneId(MilestoneId),
}

impl From<MilestoneRequest> for proto::MilestoneRequest {
    fn from(value: MilestoneRequest) -> Self {
        match value {
            MilestoneRequest::MilestoneIndex(MilestoneIndex(milestone_index)) => Self {
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
        Self::MilestoneIndex(MilestoneIndex(value.into()))
    }
}

fn to_milestone_range_request<T, I>(range: T) -> proto::MilestoneRangeRequest
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
    proto::MilestoneRangeRequest {
        start_milestone_index,
        end_milestone_index,
    }
}

/// A request for a range of milestones by [`MilestoneIndex`].
#[derive(Clone, Debug, PartialEq)]
pub struct MilestoneRangeRequest(proto::MilestoneRangeRequest);

impl<T> From<T> for MilestoneRangeRequest
where
    T: RangeBounds<u32>,
{
    fn from(value: T) -> MilestoneRangeRequest {
        MilestoneRangeRequest(to_milestone_range_request(value))
    }
}

impl From<MilestoneRangeRequest> for proto::MilestoneRangeRequest {
    fn from(value: MilestoneRangeRequest) -> Self {
        value.0
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
            MilestoneRangeRequest(proto::MilestoneRangeRequest {
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
            MilestoneRangeRequest(proto::MilestoneRangeRequest {
                start_milestone_index: 17,
                end_milestone_index: 42
            })
        );
    }
}
