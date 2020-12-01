// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::tangle::MsTangle;

use bee_message::MessageId;
use bee_storage::storage::Backend;

use log::debug;
use rand::seq::IteratorRandom;

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    time::Instant,
};

enum Score {
    NonLazy,
    SemiLazy,
    Lazy,
}

// C1: the maximum allowed delta value for the YTRSI of a given message in relation to the current LSMI before it
// gets lazy.
const YTRSI_DELTA: u32 = 8;
// C2: the maximum allowed delta value between OTRSI of a given message in relation to the current LSMI before it
// gets semi-lazy.
const OTRSI_DELTA: u32 = 13;
// M: the maximum allowed delta value between OTRSI of a given message in relation to the current LSMI before it
// gets lazy.
const BELOW_MAX_DEPTH: u32 = 15;
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

    pub(crate) async fn insert<B: Backend>(
        &mut self,
        tangle: &MsTangle<B>,
        message_id: MessageId,
        parent1: MessageId,
        parent2: MessageId,
    ) {
        if let Score::NonLazy = self.tip_score::<B>(tangle, &message_id).await {
            self.non_lazy_tips.insert(message_id);
            self.tips.insert(message_id, TipMetadata::new());
            self.link_parents_with_child(&message_id, &parent1, &parent2);
            self.check_retention_rules_for_parents(&parent1, &parent2);
        }
    }

    fn link_parents_with_child(&mut self, hash: &MessageId, parent1: &MessageId, parent2: &MessageId) {
        if parent1 == parent2 {
            self.add_child(*parent1, *hash);
        } else {
            self.add_child(*parent1, *hash);
            self.add_child(*parent2, *hash);
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

    fn check_retention_rules_for_parents(&mut self, parent1: &MessageId, parent2: &MessageId) {
        if parent1 == parent2 {
            self.check_retention_rules_for_parent(parent1);
        } else {
            self.check_retention_rules_for_parent(parent1);
            self.check_retention_rules_for_parent(parent2);
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

    pub(crate) async fn update_scores<B: Backend>(&mut self, tangle: &MsTangle<B>) {
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

    async fn tip_score<B: Backend>(&self, tangle: &MsTangle<B>, hash: &MessageId) -> Score {
        // in case the tip was pruned by the node, consider tip as lazy
        if !tangle.contains(hash).await {
            return Score::Lazy;
        }

        let lsmi = *tangle.get_latest_solid_milestone_index();
        let otrsi = *tangle.otrsi(&hash).unwrap();
        let ytrsi = *tangle.ytrsi(&hash).unwrap();

        if (lsmi - ytrsi) > YTRSI_DELTA {
            return Score::Lazy;
        }

        if (lsmi - otrsi) > BELOW_MAX_DEPTH {
            return Score::Lazy;
        }

        if (lsmi - otrsi) > OTRSI_DELTA {
            return Score::SemiLazy;
        }

        Score::NonLazy
    }

    pub fn two_non_lazy_tips(&self) -> Option<(MessageId, MessageId)> {
        let non_lazy_tips = &self.non_lazy_tips;
        if non_lazy_tips.is_empty() {
            None
        } else if non_lazy_tips.len() == 1 {
            let tip = non_lazy_tips.iter().next().unwrap();
            Some((*tip, *tip))
        } else if non_lazy_tips.len() == 2 {
            let mut iter = non_lazy_tips.iter();
            Some((*iter.next().unwrap(), *iter.next().unwrap()))
        } else {
            let hashes = non_lazy_tips.iter().choose_multiple(&mut rand::thread_rng(), 2);
            let mut iter = hashes.iter();
            Some((**iter.next().unwrap(), **iter.next().unwrap()))
        }
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
