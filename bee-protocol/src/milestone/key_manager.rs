// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::milestone::{key_range::KeyRange, MilestoneIndex};

use std::collections::HashSet;

#[derive(Clone)]
pub(crate) struct KeyManager {
    min_threshold: usize,
    key_ranges: Box<[KeyRange]>,
}

impl KeyManager {
    pub(crate) fn new(min_threshold: usize, mut key_ranges: Box<[KeyRange]>) -> Self {
        key_ranges.sort();

        Self {
            min_threshold,
            key_ranges,
        }
    }

    pub(crate) fn min_threshold(&self) -> usize {
        self.min_threshold
    }

    pub(crate) fn get_public_keys(&self, index: MilestoneIndex) -> HashSet<String> {
        let mut public_keys = HashSet::new();

        for key_range in self.key_ranges.iter() {
            if key_range.start() <= index {
                if key_range.end() >= index || key_range.start() == key_range.end() {
                    public_keys.insert(key_range.public_key().clone());
                }
                continue;
            }
            break;
        }

        public_keys
    }
}

#[test]
fn key_manager_sort() {
    let krs = vec![
        KeyRange::new("kr1".to_string(), 42.into(), 1000.into()),
        KeyRange::new("kr2".to_string(), 21.into(), 1000.into()),
        KeyRange::new("kr3".to_string(), 84.into(), 1000.into()),
        KeyRange::new("kr4".to_string(), 0.into(), 1000.into()),
    ];

    let km = KeyManager::new(0, krs.into_boxed_slice());

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
