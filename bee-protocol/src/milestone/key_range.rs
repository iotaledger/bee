// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::milestone::MilestoneIndex;

use serde::Deserialize;

use std::cmp::Ordering;

#[derive(Clone, Deserialize, Eq, PartialEq)]
pub struct KeyRange {
    // TODO ED25 pk
    public_key: String,
    start: MilestoneIndex,
    end: MilestoneIndex,
}

impl Ord for KeyRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for KeyRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl KeyRange {
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
