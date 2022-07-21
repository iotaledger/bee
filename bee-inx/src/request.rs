// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Bound, RangeBounds};

use inx::proto;

#[derive(Clone, Debug, PartialEq)]
pub struct MilestoneRangeRequest(proto::MilestoneRangeRequest);

pub(crate) fn to_milestone_range_request<T, I>(range: T) -> proto::MilestoneRangeRequest
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

impl<T> From<T> for MilestoneRangeRequest
where
    T: RangeBounds<u32>,
{
    fn from(value: T) -> MilestoneRangeRequest {
        let start_milestone_index = match value.start_bound() {
            Bound::Included(&idx) => idx,
            Bound::Excluded(&idx) => idx + 1,
            Bound::Unbounded => 0,
        };
        let end_milestone_index = match value.end_bound() {
            Bound::Included(&idx) => idx,
            Bound::Excluded(&idx) => idx - 1,
            Bound::Unbounded => 0,
        };
        MilestoneRangeRequest(proto::MilestoneRangeRequest {
            start_milestone_index,
            end_milestone_index,
        })
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
        let from = 17u32;
        let to = 42;
        let range = MilestoneRangeRequest::from(from..=to);
        assert_eq!(
            range,
            MilestoneRangeRequest(proto::MilestoneRangeRequest {
                start_milestone_index: 17,
                end_milestone_index: 42
            })
        );
    }
}
