// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::{Add, Sub};

use derive_more::{Deref, From};

/// A wrapper around a `u32` that represents a milestone index.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, From, Deref, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestoneIndex(pub u32);

impl MilestoneIndex {
    /// Creates a new `MilestoneIndex`.
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

impl core::fmt::Display for MilestoneIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
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
