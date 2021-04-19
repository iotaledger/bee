// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a milestone key range manager type.

use crate::types::milestone_key_range::MilestoneKeyRange;

use bee_message::milestone::MilestoneIndex;

use std::collections::HashSet;

/// A key manager is managing a set of `MilestoneKeyRange`s.
#[derive(Clone)]
pub struct MilestoneKeyManager {
    min_threshold: usize,
    key_ranges: Box<[MilestoneKeyRange]>,
}

impl MilestoneKeyManager {
    /// Creates a new `MilestoneKeyManager`.
    pub fn new(min_threshold: usize, mut key_ranges: Box<[MilestoneKeyRange]>) -> Self {
        key_ranges.sort();

        Self {
            min_threshold,
            key_ranges,
        }
    }

    /// Returns the minimum threshold of the `MilestoneKeyManager`.
    pub fn min_threshold(&self) -> usize {
        self.min_threshold
    }

    /// Returns a set of public keys applicable for a given milestone index.
    pub fn get_public_keys(&self, index: MilestoneIndex) -> HashSet<String> {
        let mut public_keys = HashSet::with_capacity(self.key_ranges.len());

        for key_range in self.key_ranges.iter() {
            if key_range.start() <= index {
                if key_range.end() >= index
                // start == end means the key is valid forever.
                || key_range.start() == key_range.end()
                {
                    public_keys.insert(key_range.public_key().clone());
                }
                continue;
            }
            break;
        }

        public_keys
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn key_manager_is_sorted() {
        let krs = vec![
            MilestoneKeyRange::new("kr1".to_string(), 42.into(), 1000.into()),
            MilestoneKeyRange::new("kr2".to_string(), 21.into(), 1000.into()),
            MilestoneKeyRange::new("kr3".to_string(), 84.into(), 1000.into()),
            MilestoneKeyRange::new("kr4".to_string(), 0.into(), 1000.into()),
        ];

        let km = MilestoneKeyManager::new(0, krs.into_boxed_slice());

        assert_eq!(km.key_ranges[0].public_key(), "kr4");
        assert_eq!(km.key_ranges[0].start(), 0.into());
        assert_eq!(km.key_ranges[0].end(), 1000.into());

        assert_eq!(km.key_ranges[1].public_key(), "kr2");
        assert_eq!(km.key_ranges[1].start(), 21.into());
        assert_eq!(km.key_ranges[1].end(), 1000.into());

        assert_eq!(km.key_ranges[2].public_key(), "kr1");
        assert_eq!(km.key_ranges[2].start(), 42.into());
        assert_eq!(km.key_ranges[2].end(), 1000.into());

        assert_eq!(km.key_ranges[3].public_key(), "kr3");
        assert_eq!(km.key_ranges[3].start(), 84.into());
        assert_eq!(km.key_ranges[3].end(), 1000.into());
    }
}
