// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a milestone key range type.

use bee_message::milestone::MilestoneIndex;

use core::cmp::Ordering;

/// A milestone key range is a milestone public key valid for a given interval of milestones.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestoneKeyRange {
    public_key: String,
    // Inclusive bound.
    start: MilestoneIndex,
    // Inclusive bound.
    end: MilestoneIndex,
}

impl MilestoneKeyRange {
    /// Creates a new `MilestoneKeyRange`.
    pub fn new(public_key: String, start: MilestoneIndex, end: MilestoneIndex) -> Self {
        Self { public_key, start, end }
    }

    /// Returns the public key of the `MilestoneKeyRange`.
    pub fn public_key(&self) -> &String {
        &self.public_key
    }

    /// Returns the start index of the `MilestoneKeyRange`.
    pub fn start(&self) -> MilestoneIndex {
        self.start
    }

    /// Return the end index of the `MilestoneKeyRange`.
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
