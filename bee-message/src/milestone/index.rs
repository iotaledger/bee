// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::Packable;

use core::ops::{Add, Deref, Sub};

/// A wrapper around a `u32` that represents a milestone index.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestoneIndex(pub u32);

impl MilestoneIndex {
    /// Creates a new `MilestoneIndex`.
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for MilestoneIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for MilestoneIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for MilestoneIndex {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl Add for MilestoneIndex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(*self + *other)
    }
}

impl Add<u32> for MilestoneIndex {
    type Output = Self;

    fn add(self, other: u32) -> Self {
        Self(*self + other)
    }
}

impl Sub for MilestoneIndex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(*self - *other)
    }
}

impl Sub<u32> for MilestoneIndex {
    type Output = Self;

    fn sub(self, other: u32) -> Self {
        Self(*self - other)
    }
}
