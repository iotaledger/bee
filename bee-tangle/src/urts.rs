// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{ms_tangle::MsTangle, storage::StorageBackend};

use bee_message::MessageId;

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use log::debug;
use rand::seq::IteratorRandom;

use std::time::Instant;

enum Score {
    NonLazy,
    SemiLazy,
    Lazy,
}

// C1: the maximum allowed delta value for the YMRSI of a given message in relation to the current SMI before it
// gets lazy.
const YMRSI_DELTA: u32 = 8;
// C2: the maximum allowed delta value between OMRSI of a given message in relation to the current SMI before it
// gets semi-lazy.
const OMRSI_DELTA: u32 = 13;
// M: the maximum allowed delta value between OMRSI of a given message in relation to the current SMI before it
// gets lazy.
pub const BELOW_MAX_DEPTH: u32 = 15;
// If the amount of non-lazy tips exceed this limit, remove the parent(s) of the inserted tip to compensate for the
// excess. This rule helps to reduce the amount of tips in the network.
const MAX_LIMIT_NON_LAZY: u8 = 100;
// The maximum time a tip remains in the tip pool after having the first child.
// This rule helps to widen the tangle.
const MAX_AGE_SECONDS_AFTER_FIRST_CHILD: u8 = 3;
// The maximum amount of children a tip is allowed to have before the tip is removed from the tip pool. This rule is
// used to widen the cone of the tangle.
const MAX_NUM_CHILDREN: u8 = 2;

#[derive(Default)]
struct TipMetadata {
    children: HashSet<MessageId>,
    time_first_child: Option<Instant>,
}

impl TipMetadata {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub(crate) struct UrtsTipPool {
    tips: HashMap<MessageId, TipMetadata>,
    non_lazy_tips: HashSet<MessageId>,
}

impl UrtsTipPool {
    pub(crate) fn non_lazy_tips(&self) -> &HashSet<MessageId> {
        &self.non_lazy_tips
    }

    pub(crate) async fn insert<B: StorageBackend>(
        &mut self,
        tangle: &MsTangle<B>,
        message_id: MessageId,
        parents: Vec<MessageId>,
    ) {
        if let Score::NonLazy = self.tip_score::<B>(tangle, &message_id).await {
            self.non_lazy_tips.insert(message_id);
            self.tips.insert(message_id, TipMetadata::new());
            for parent in parents.iter() {
                self.add_child(*parent, message_id);
                self.check_retention_rules_for_parent(parent);
            }
        }
    }

    fn add_child(&mut self, parent: MessageId, child: MessageId) {
        match self.tips.entry(parent) {
            Entry::Occupied(mut entry) => {
                let metadata = entry.get_mut();
                metadata.children.insert(child);
                if metadata.time_first_child == None {
                    metadata.time_first_child = Some(Instant::now());
                }
            }
            Entry::Vacant(entry) => {
                let mut metadata = TipMetadata::new();
                metadata.children.insert(child);
                metadata.time_first_child = Some(Instant::now());
                entry.insert(metadata);
            }
        }
    }

    fn check_retention_rules_for_parent(&mut self, parent: &MessageId) {
        if self.non_lazy_tips.len() > MAX_LIMIT_NON_LAZY as usize
            || self.tips.get(parent).unwrap().children.len() as u8 > MAX_NUM_CHILDREN
            || self
                .tips
                .get(parent)
                .unwrap()
                .time_first_child
                .unwrap()
                .elapsed()
                .as_secs() as u8
                > MAX_AGE_SECONDS_AFTER_FIRST_CHILD
        {
            self.tips.remove(parent);
            self.non_lazy_tips.remove(parent);
        }
    }

    pub(crate) async fn update_scores<B: StorageBackend>(&mut self, tangle: &MsTangle<B>) {
        let mut to_remove = Vec::new();

        for tip in self.tips.keys() {
            match self.tip_score::<B>(tangle, &tip).await {
                Score::SemiLazy | Score::Lazy => {
                    to_remove.push(*tip);
                }
                _ => continue,
            }
        }

        for tip in to_remove {
            self.tips.remove(&tip);
            self.non_lazy_tips.remove(&tip);
        }

        debug!("Non-lazy tips {}", self.non_lazy_tips.len());
    }

    async fn tip_score<B: StorageBackend>(&self, tangle: &MsTangle<B>, hash: &MessageId) -> Score {
        // in case the tip was pruned by the node, consider tip as lazy
        if !tangle.contains(hash).await {
            return Score::Lazy;
        }

        let smi = *tangle.get_solid_milestone_index();
        let omrsi = *tangle.omrsi(&hash).await.unwrap().index();
        let ymrsi = *tangle.ymrsi(&hash).await.unwrap().index();

        if (smi - ymrsi) > YMRSI_DELTA {
            return Score::Lazy;
        }

        if (smi - omrsi) > BELOW_MAX_DEPTH {
            return Score::Lazy;
        }

        if (smi - omrsi) > OMRSI_DELTA {
            return Score::SemiLazy;
        }

        Score::NonLazy
    }

    pub fn two_non_lazy_tips(&self) -> Option<Vec<MessageId>> {
        let non_lazy_tips = &self.non_lazy_tips;
        if non_lazy_tips.is_empty() {
            None
        } else {
            Some(if non_lazy_tips.len() < self.optimal_num_tips() {
                non_lazy_tips.iter().copied().collect()
            } else {
                non_lazy_tips
                    .iter()
                    .choose_multiple(&mut rand::thread_rng(), self.optimal_num_tips())
                    .iter()
                    .map(|t| **t)
                    .collect()
            })
        }
    }

    pub(crate) fn optimal_num_tips(&self) -> usize {
        // TODO: hardcoded at the moment
        4
    }

    pub(crate) fn reduce_tips(&mut self) {
        let mut to_remove = Vec::new();
        for (tip, metadata) in &self.tips {
            if let Some(age) = metadata.time_first_child {
                if age.elapsed().as_secs() as u8 > MAX_AGE_SECONDS_AFTER_FIRST_CHILD {
                    to_remove.push(*tip);
                }
            }
        }
        for tip in to_remove {
            self.tips.remove(&tip);
            self.non_lazy_tips.remove(&tip);
        }
    }
}
