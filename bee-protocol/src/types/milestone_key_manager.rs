// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a milestone key range manager type.

use bee_message::milestone::MilestoneIndex;

use crate::types::milestone_key_range::MilestoneKeyRange;

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

    /// Returns the key ranges that are applicable for a given milestone index.
    pub fn applicable_key_ranges(&self, index: MilestoneIndex) -> impl Iterator<Item = &MilestoneKeyRange> {
        self.key_ranges
            .iter()
            .take_while(move |key_range| key_range.start() <= index)
            .filter(move |key_range| key_range.end() >= index || *key_range.end() == 0)
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use super::*;

    #[test]
    fn key_manager_is_sorted() {
        let krs = vec![
            MilestoneKeyRange::new("kr0".to_string(), 42.into(), 1000.into()),
            MilestoneKeyRange::new("kr1".to_string(), 21.into(), 1000.into()),
            MilestoneKeyRange::new("kr2".to_string(), 84.into(), 1000.into()),
            MilestoneKeyRange::new("kr3".to_string(), 0.into(), 1000.into()),
        ];

        let km = MilestoneKeyManager::new(0, krs.into_boxed_slice());

        assert_eq!(km.key_ranges[0].public_key(), "kr3");
        assert_eq!(km.key_ranges[0].start(), 0.into());
        assert_eq!(km.key_ranges[0].end(), 1000.into());

        assert_eq!(km.key_ranges[1].public_key(), "kr1");
        assert_eq!(km.key_ranges[1].start(), 21.into());
        assert_eq!(km.key_ranges[1].end(), 1000.into());

        assert_eq!(km.key_ranges[2].public_key(), "kr0");
        assert_eq!(km.key_ranges[2].start(), 42.into());
        assert_eq!(km.key_ranges[2].end(), 1000.into());

        assert_eq!(km.key_ranges[3].public_key(), "kr2");
        assert_eq!(km.key_ranges[3].start(), 84.into());
        assert_eq!(km.key_ranges[3].end(), 1000.into());
    }

    #[test]
    fn applicable_key_ranges() {
        let krs = vec![
            MilestoneKeyRange::new("kr0".to_string(), 0.into(), 50.into()), // Does not apply
            MilestoneKeyRange::new("kr1".to_string(), 0.into(), 25.into()), // Does not apply
            MilestoneKeyRange::new("kr2".to_string(), 0.into(), 100.into()), // Applies
            MilestoneKeyRange::new("kr3".to_string(), 25.into(), 0.into()), // Applies
            MilestoneKeyRange::new("kr4".to_string(), 50.into(), 75.into()), // Applies
            MilestoneKeyRange::new("kr5".to_string(), 50.into(), 150.into()), // Applies
            MilestoneKeyRange::new("kr6".to_string(), 75.into(), 175.into()), // Applies
            MilestoneKeyRange::new("kr7".to_string(), 100.into(), 200.into()), // Does not apply
            MilestoneKeyRange::new("kr8".to_string(), 125.into(), 225.into()), // Does not apply
        ];

        let km = MilestoneKeyManager::new(0, krs.into_boxed_slice());
        let public_keys = km
            .applicable_key_ranges(MilestoneIndex(75))
            .map(|kr| kr.public_key())
            .cloned()
            .collect::<HashSet<String>>();

        assert_eq!(public_keys.len(), 5);
        assert!(public_keys.contains("kr2"));
        assert!(public_keys.contains("kr3"));
        assert!(public_keys.contains("kr4"));
        assert!(public_keys.contains("kr5"));
        assert!(public_keys.contains("kr6"));
    }
}
