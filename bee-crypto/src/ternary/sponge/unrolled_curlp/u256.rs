// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Index, IndexMut};

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

impl U256 {
    pub(super) fn bit(&self, i: usize) -> u64 {
        self[(i / 64) % 4] >> (i % 64) & 1
    }

    pub(super) fn set_bit(&mut self, i: usize) {
        self[(i / 64) % 4] |= 1 << (i % 64)
    }

    pub(super) fn shr_into(&mut self, x: &Self, shift: usize) -> &mut Self {
        let offset = shift / 64;
        let r = shift % 64;

        if r == 0 {
            for i in offset..4 {
                self[(i - offset) % 4] |= x[i];
            }
            return self;
        }

        let mut l = 64 - r;
        l &= 63;

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
            _ => {}
        }

        self
    }

    pub(super) fn shl_into(&mut self, x: &Self, shift: usize) -> &mut Self {
        let offset = shift / 64;
        let l = shift % 64;

        if l == 0 {
            for i in offset..4 {
                self[i] |= x[(i - offset) % 4];
            }
            return self;
        }

        let mut r = 64 - l;
        r &= 63;

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
            _ => {}
        }

        self
    }

    pub(super) fn norm243(&mut self) {
        self[3] &= (1 << (64 - (256 - 243))) - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::U256;

    #[test]
    fn get_bits() {
        let x = U256([1, 0, 0, 0]);

        assert_eq!(x.bit(0), 1, "the first bit should be one");

        for i in 1..256 {
            assert_eq!(x.bit(i), 0, "bit {} should be zero", i);
        }
    }

    #[test]
    fn set_bits() {
        let mut x = U256::default();
        x.set_bit(42);

        assert_eq!(x.bit(42), 1, "the 42th bit should be one");

        for i in (0..42).chain(43..256) {
            assert_eq!(x.bit(i), 0, "bit {} should be zero", i);
        }
    }

    #[test]
    fn shr_into() {
        let mut x = U256([1, 2, 3, 4]);
        let y = U256([5, 6, 7, 8]);

        assert_eq!(
            U256([216172782113783809, 252201579132747778, 288230376151711747, 4]),
            *x.shr_into(&y, 9)
        );
    }

    #[test]
    fn shl_into() {
        let mut x = U256([1, 2, 3, 4]);
        let y = U256([5, 6, 7, 8]);

        assert_eq!(U256([2561, 3074, 3587, 4100]), *x.shl_into(&y, 9));
    }

    #[test]
    fn norm243() {
        let mut x = U256([u64::MAX; 4]);
        x.norm243();

        assert_eq!(U256([u64::MAX, u64::MAX, u64::MAX, 2251799813685247]), x);
    }
}
