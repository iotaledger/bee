// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Extensions to `overflowing_add`.

pub(crate) trait OverflowingAdd<T = Self> {
    /// Extends `overflowing_add` with a carry.
    fn overflowing_add_with_carry(self, other: T, carry: T) -> (T, bool);
}

impl OverflowingAdd for u32 {
    fn overflowing_add_with_carry(self, other: Self, carry: Self) -> (Self, bool) {
        let (sum, first_overflow) = self.overflowing_add(other);
        let (sum, second_overflow) = sum.overflowing_add(carry);

        (sum, first_overflow | second_overflow)
    }
}
