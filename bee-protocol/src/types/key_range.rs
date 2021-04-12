// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;

use core::cmp::Ordering;

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestoneKeyRange {
    // TODO ED25 pk
    public_key: String,
    start: MilestoneIndex,
    end: MilestoneIndex,
}

impl MilestoneKeyRange {
    pub fn new(public_key: String, start: MilestoneIndex, end: MilestoneIndex) -> Self {
        Self { public_key, start, end }
    }

    pub fn public_key(&self) -> &String {
        &self.public_key
    }

    pub fn start(&self) -> MilestoneIndex {
        self.start
    }

    pub fn end(&self) -> MilestoneIndex {
        self.end
    }
}

impl Ord for MilestoneKeyRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for MilestoneKeyRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
