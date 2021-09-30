// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Div, Sub};

#[repr(transparent)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub(super) struct BoundedUsize<const N: usize>(usize);

impl<const N: usize> BoundedUsize<N> {
    /// SAFETY: `value` should be smaller than `N`.
    pub(super) unsafe fn from_usize_unchecked(value: usize) -> Self {
        debug_assert!(value < N);
        Self(value)
    }

    pub(super) fn from_usize(value: usize) -> Option<Self> {
        if value < N { Some(Self(value)) } else { None }
    }

    pub(super) fn into_usize(self) -> usize {
        self.0
    }
}

impl BoundedUsize<256> {
    pub(super) const C64: Self = Self(64);
    pub(super) const C243: Self = Self(243);
}

impl<const N: usize> Sub for BoundedUsize<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl<const N: usize> Div for BoundedUsize<N> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
