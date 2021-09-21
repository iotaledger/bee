// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use u256::U256;

use super::{Sponge, HASH_LENGTH};

use bee_ternary::{Btrit, Trits};

use std::convert::{Infallible, TryFrom};

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
        Self {
            p: Default::default(),
            n: Default::default(),
            direction: SpongeDirection::Absorb,
        }
    }
}

impl UnrolledCurlP81 {
    pub fn copy_state(&self, mut s: &mut Trits) {
        for i in 0..3 {
            let _ = s.get(HASH_LENGTH - 1).unwrap();

            for j in 0..HASH_LENGTH {
                if self.p[i].bit(j) != 0 {
                    s.set(j, Btrit::PlusOne);
                } else if self.n[i].bit(j) != 0 {
                    s.set(j, Btrit::NegOne);
                } else {
                    s.set(j, Btrit::Zero);
                }
            }
            s = &mut s[HASH_LENGTH..];
        }
    }

    fn squeeze_aux(&mut self, mut hash: &mut Trits) {
        if let SpongeDirection::Squeeze = self.direction {
            self.transform();
        }

        self.direction = SpongeDirection::Squeeze;

        hash = &mut hash[..HASH_LENGTH];

        for i in 0..HASH_LENGTH {
            hash.set(i, Btrit::try_from((self.p[0].bit(i) - self.n[0].bit(i)) as u8).unwrap());
        }
    }

    fn transform(&mut self) {
        transform::transform(&mut self.p, &mut self.n)
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
        assert_ne!(buf.len() % HASH_LENGTH, 0, "Invalid squeeze length");

        while buf.len() >= HASH_LENGTH {
            self.squeeze_aux(buf);
            buf = &mut buf[HASH_LENGTH..];
        }

        Ok(())
    }
}

mod u256 {
    use std::ops::{Index, IndexMut};

    #[derive(Clone, Copy)]
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
                    self[0] |= x[3] >> r;
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
            let r = shift % 64;

            let mut l = 64 - r;
            l &= 64;

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

        pub(super) fn norm243(&mut self) {
            self[3] &= 1 << (64 - (256 - 243)) - 1;
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

    fn rotate_state(p: &mut [U256; 3], n: &mut [U256; 3], offset: usize, shift: usize) -> ([U256; 3], [U256; 3]) {
        let mut p2 = <[U256; 3]>::default();
        let mut n2 = <[U256; 3]>::default();

        macro_rules! rotate {
            ($part:expr, $part2:expr, $i:expr) => {
                $part2[$i]
                    .shr_into(&$part[($i + offset) % 3], shift)
                    .shl_into(&$part[($i + offset) % 3], 243 - shift);
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
                ($part:expr, $part2:expr, $j:expr) => {
                    $part2[i][$j] = ($part[i][$j] & M0) | ($part[(1 + i) % 3][$j] & M1) | ($part[(2 + i) % 3][$j] & M2);
                };
            }

            compute!(p, p2, 0);
            compute!(p, p2, 1);
            compute!(p, p2, 2);
            compute!(p, p2, 3);

            compute!(n, n2, 0);
            compute!(n, n2, 1);
            compute!(n, n2, 2);
            compute!(n, n2, 3);
        }

        *p = p2;
        *n = n2;
    }
}
