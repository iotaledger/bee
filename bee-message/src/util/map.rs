// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use alloc::vec::Vec;
use core::cmp::Ord;

#[repr(transparent)]
pub(crate) struct Map<K, V> {
    inner: Vec<(K, V)>,
}

impl<K, V> Map<K, V> {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    #[inline(always)]
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }
}

impl<K: Ord, V> Map<K, V> {
    pub(crate) fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self.inner.binary_search_by_key(&&k, |x| &x.0) {
            Ok(index) => {
                let (_, old_v) = core::mem::replace(&mut self.inner[index], (k, v));
                Some(old_v)
            }
            Err(index) => {
                self.inner.insert(index, (k, v));
                None
            }
        }
    }
}

#[repr(transparent)]
pub(crate) struct Set<T> {
    inner: Map<T, ()>,
}

impl<T> Set<T> {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self { inner: Map::new() }
    }

    #[inline(always)]
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Map::with_capacity(cap),
        }
    }
}

impl<T: Ord> Set<T> {
    #[inline(always)]
    pub(crate) fn insert(&mut self, t: T) -> bool {
        self.inner.insert(t, ()).is_none()
    }
}
