// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cmp::Eq, ops::Deref};

#[derive(Clone)]
pub struct VecSet<T> {
    items: Vec<T>,
}

impl<T> Default for VecSet<T> {
    fn default() -> Self {
        Self { items: Vec::default() }
    }
}

impl<T> VecSet<T> {
    pub fn insert(&mut self, item: T) -> bool
    where
        T: Eq,
    {
        if self.items.contains(&item) {
            false
        } else {
            self.items.push(item);
            true
        }
    }
}

impl<T> Deref for VecSet<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
