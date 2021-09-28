// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::bounded::BoundedUsize;

use std::{
    hint::unreachable_unchecked,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct U256([u64; 4]);

impl Default for U256 {
    fn default() -> Self {
        Self([u64::default(); 4])
    }
}

impl Index<usize> for U256 {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for U256 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl Index<BoundedUsize<256>> for U256 {
    type Output = u64;

    fn index(&self, index: BoundedUsize<256>) -> &Self::Output {
        // SAFETY: index is smaller than 256.
        unsafe { self.get_unchecked(index.into_usize()) }
    }
}

impl IndexMut<BoundedUsize<256>> for U256 {
    fn index_mut(&mut self, index: BoundedUsize<256>) -> &mut Self::Output {
        // SAFETY: index is smaller than 256.
        unsafe { self.get_unchecked_mut(index.into_usize()) }
    }
}

impl U256 {
    unsafe fn get_unchecked(&self, index: usize) -> &u64 {
        self.0.get_unchecked(index)
    }

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut u64 {
        self.0.get_unchecked_mut(index)
    }

    // Doing a bitwise AND with `1` produces a value between `0` and `1` which fits in an `i8` without truncation.
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn bit(&self, i: BoundedUsize<256>) -> i8 {
        (self[i / BoundedUsize::C64] >> (i.into_usize() % 64) & 1) as i8
    }

    pub(super) fn set_bit(&mut self, i: BoundedUsize<256>) {
        self[i / BoundedUsize::C64] |= 1 << (i.into_usize() % 64)
    }

    pub(super) fn shr_into(&mut self, x: &Self, shift: BoundedUsize<256>) -> &mut Self {
        // `offset` is smaller than `4`.
        let offset = shift.into_usize() / 64;
        let r = shift.into_usize() % 64;

        if r == 0 {
            for i in offset..4 {
                // SAFETY: `i` is between `offset` and `4` and `offset` is smaller than `4`.
                unsafe {
                    *self.get_unchecked_mut(i - offset) |= *x.get_unchecked(i);
                }
            }
            return self;
        }

        let l = 64 - r;

        match offset {
            0 => {
                self[0] |= x[0] >> r | x[1] << l;
                self[1] |= x[1] >> r | x[2] << l;
                self[2] |= x[2] >> r | x[3] << l;
                self[3] |= x[3] >> r;
            }
            1 => {
                self[0] |= x[1] >> r | x[2] << l;
                self[1] |= x[2] >> r | x[3] << l;
                self[2] |= x[3] >> r;
            }
            2 => {
                self[0] |= x[2] >> r | x[3] << l;
                self[1] |= x[3] >> r;
            }
            3 => {
                self[0] |= x[3] >> r;
            }
            // SAFETY: `offset` is never greater or equal than 4.
            _ => unsafe { unreachable_unchecked() },
        }

        self
    }

    pub(super) fn shl_into(&mut self, x: &Self, shift: BoundedUsize<256>) -> &mut Self {
        // `offset` is smaller than `4`.
        let offset = shift.into_usize() / 64;
        let l = shift.into_usize() % 64;

        if l == 0 {
            for i in offset..4 {
                // SAFETY: `i` is between `offset` and `4` and `offset` is smaller than `4`.
                unsafe {
                    *self.get_unchecked_mut(i) |= *x.get_unchecked(i - offset);
                }
            }
            return self;
        }

        let r = 64 - l;

        match offset {
            0 => {
                self[3] |= x[3] << l | x[2] >> r;
                self[2] |= x[2] << l | x[1] >> r;
                self[1] |= x[1] << l | x[0] >> r;
                self[0] |= x[0] << l;
            }
            1 => {
                self[3] |= x[2] << l | x[1] >> r;
                self[2] |= x[1] << l | x[0] >> r;
                self[1] |= x[0] << l;
            }
            2 => {
                self[3] |= x[1] << l | x[0] >> r;
                self[2] |= x[0] << l;
            }
            3 => {
                self[3] |= x[0] << l;
            }
            // SAFETY: `offset` is never greater or equal than 4.
            _ => unsafe { unreachable_unchecked() },
        }

        self
    }

    pub(super) fn norm243(&mut self) {
        self[3] &= (1 << (64 - (256 - 243))) - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundedUsize, U256};

    #[test]
    fn get_bits() {
        let x = U256([1, 0, 0, 0]);

        assert_eq!(
            x.bit(BoundedUsize::from_usize(0).unwrap()),
            1,
            "the first bit should be one"
        );

        for i in 1..256 {
            assert_eq!(
                x.bit(BoundedUsize::from_usize(i).unwrap()),
                0,
                "bit {} should be zero",
                i
            );
        }
    }

    #[test]
    fn set_bits() {
        let mut x = U256::default();
        x.set_bit(BoundedUsize::from_usize(42).unwrap());

        assert_eq!(
            x.bit(BoundedUsize::from_usize(42).unwrap()),
            1,
            "the 42th bit should be one"
        );

        for i in (0..42).chain(43..256) {
            assert_eq!(
                x.bit(BoundedUsize::from_usize(i).unwrap()),
                0,
                "bit {} should be zero",
                i
            );
        }
    }

    #[test]
    fn shr_into() {
        let mut x = U256([1, 2, 3, 4]);
        let y = U256([5, 6, 7, 8]);

        assert_eq!(
            U256([216172782113783809, 252201579132747778, 288230376151711747, 4]),
            *x.shr_into(&y, BoundedUsize::from_usize(9).unwrap())
        );
    }

    #[test]
    fn shl_into() {
        let mut x = U256([1, 2, 3, 4]);
        let y = U256([5, 6, 7, 8]);

        assert_eq!(
            U256([2561, 3074, 3587, 4100]),
            *x.shl_into(&y, BoundedUsize::from_usize(9).unwrap())
        );
    }

    #[test]
    fn norm243() {
        let mut x = U256([u64::MAX; 4]);
        x.norm243();

        assert_eq!(U256([u64::MAX, u64::MAX, u64::MAX, 2251799813685247]), x);
    }
}
