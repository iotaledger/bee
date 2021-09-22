// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use u256::U256;

use super::{Sponge, HASH_LENGTH};

use bee_ternary::{Btrit, Trits};

use std::convert::Infallible;

enum SpongeDirection {
    Absorb,
    Squeeze,
}

pub struct UnrolledCurlP81 {
    p: [U256; 3],
    n: [U256; 3],
    direction: SpongeDirection,
}

impl UnrolledCurlP81 {
    pub fn new() -> Self {
        Self::default()
    }

    fn squeeze_aux(&mut self, mut hash: &mut Trits) {
        if let SpongeDirection::Squeeze = self.direction {
            self.transform();
        }

        self.direction = SpongeDirection::Squeeze;

        hash = &mut hash[..HASH_LENGTH];

        for i in 0..HASH_LENGTH {
            let trit = match (self.p[0].bit(i), self.n[0].bit(i)) {
                (a, b) if a > b => Btrit::PlusOne,
                (a, b) if a < b => Btrit::NegOne,
                _ => Btrit::Zero,
            };

            hash.set(i, trit);
        }
    }

    fn transform(&mut self) {
        transform::transform(&mut self.p, &mut self.n)
    }
}

impl Default for UnrolledCurlP81 {
    fn default() -> Self {
        Self {
            p: Default::default(),
            n: Default::default(),
            direction: SpongeDirection::Absorb,
        }
    }
}

impl Sponge for UnrolledCurlP81 {
    type Error = Infallible;

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn absorb(&mut self, mut input: &Trits) -> Result<(), Self::Error> {
        if input.len() == 0 || input.len() % HASH_LENGTH != 0 {
            panic!("trits slice length must be multiple of {}", HASH_LENGTH);
        }

        if let SpongeDirection::Squeeze = self.direction {
            panic!("absorb after squeeze");
        }

        while input.len() >= HASH_LENGTH {
            let mut p = U256::default();
            let mut n = U256::default();

            for i in 0..HASH_LENGTH {
                match input[i] {
                    Btrit::PlusOne => p.set_bit(i),
                    Btrit::Zero => (),
                    Btrit::NegOne => n.set_bit(i),
                }
            }

            self.p[0] = p;
            self.n[0] = n;
            input = &input[HASH_LENGTH..];
            self.transform();
        }

        Ok(())
    }

    fn squeeze_into(&mut self, mut buf: &mut Trits) -> Result<(), Self::Error> {
        assert_eq!(buf.len() % HASH_LENGTH, 0, "Invalid squeeze length");

        while {
            self.squeeze_aux(buf);
            buf = &mut buf[HASH_LENGTH..];
            buf.len() >= HASH_LENGTH
        } {}

        Ok(())
    }
}

mod u256 {
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
}

mod transform {
    const NUM_ROUNDS: usize = 81;
    const ROTATION_OFFSET: usize = 364;
    const STATE_SIZE: usize = HASH_LENGTH * 3;

    use super::{u256::U256, HASH_LENGTH};

    use lazy_static::lazy_static;

    #[derive(Clone, Copy)]
    struct StateRotation {
        offset: usize,
        shift: usize,
    }

    lazy_static! {
        static ref STATE_ROTATIONS: [StateRotation; NUM_ROUNDS] = {
            let mut rotation = ROTATION_OFFSET;

            let mut state_rotations = [StateRotation { offset: 0, shift: 0 }; NUM_ROUNDS];

            for state_rotation in &mut state_rotations {
                state_rotation.offset = rotation / HASH_LENGTH;
                state_rotation.shift = rotation % HASH_LENGTH;
                rotation = (rotation * ROTATION_OFFSET) % STATE_SIZE;
            }

            state_rotations
        };
    }

    pub(super) fn transform(p: &mut [U256; 3], n: &mut [U256; 3]) {
        for state_rotation in STATE_ROTATIONS.iter() {
            let (p2, n2) = rotate_state(p, n, state_rotation.offset, state_rotation.shift);

            macro_rules! compute {
                ($i: expr, $j: expr) => {
                    let tmp = batch_box(p[$i][$j], n[$i][$j], p2[$i][$j], n2[$i][$j]);
                    p[$i][$j] = tmp.0;
                    n[$i][$j] = tmp.1;
                };
            }

            compute!(0, 0);
            compute!(0, 1);
            compute!(0, 2);
            compute!(0, 3);
            compute!(1, 0);
            compute!(1, 1);
            compute!(1, 2);
            compute!(1, 3);
            compute!(2, 0);
            compute!(2, 1);
            compute!(2, 2);
            compute!(2, 3);

            p[0].norm243();
            p[1].norm243();
            p[2].norm243();
            n[0].norm243();
            n[1].norm243();
            n[2].norm243();
        }

        reorder(p, n);
    }

    fn rotate_state(p: &[U256; 3], n: &[U256; 3], offset: usize, shift: usize) -> ([U256; 3], [U256; 3]) {
        let mut p2 = <[U256; 3]>::default();
        let mut n2 = <[U256; 3]>::default();

        macro_rules! rotate {
            ($p:expr, $p2:expr, $i:expr) => {
                $p2[$i]
                    .shr_into(&$p[($i + offset) % 3], shift)
                    .shl_into(&$p[(($i + 1) + offset) % 3], 243 - shift);
            };
        }

        rotate!(p, p2, 0);
        rotate!(p, p2, 1);
        rotate!(p, p2, 2);

        rotate!(n, n2, 0);
        rotate!(n, n2, 1);
        rotate!(n, n2, 2);

        (p2, n2)
    }

    fn batch_box(x_p: u64, x_n: u64, y_p: u64, y_n: u64) -> (u64, u64) {
        let tmp = x_n ^ y_p;
        return (tmp & !x_p, !tmp & !(x_p ^ y_n));
    }

    fn reorder(p: &mut [U256; 3], n: &mut [U256; 3]) {
        const M0: u64 = 0x9249249249249249;
        const M1: u64 = (M0 << 1) & u64::MAX;
        const M2: u64 = (M0 << 2) & u64::MAX;

        let mut p2 = <[U256; 3]>::default();
        let mut n2 = <[U256; 3]>::default();

        for i in 0..3 {
            macro_rules! compute {
                ($p:expr, $p2:expr, $j:expr, $m0:expr, $m1:expr, $m2:expr) => {
                    $p2[i][$j] = ($p[i][$j] & $m0) | ($p[(1 + i) % 3][$j] & $m1) | ($p[(2 + i) % 3][$j] & $m2);
                };
            }

            compute!(p, p2, 0, M0, M1, M2);
            compute!(p, p2, 1, M2, M0, M1);
            compute!(p, p2, 2, M1, M2, M0);
            compute!(p, p2, 3, M0, M1, M2);

            compute!(n, n2, 0, M0, M1, M2);
            compute!(n, n2, 1, M2, M0, M1);
            compute!(n, n2, 2, M1, M2, M0);
            compute!(n, n2, 3, M0, M1, M2);
        }

        *p = p2;
        *n = n2;
    }
}
