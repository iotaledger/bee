// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{u256::U256, HASH_LENGTH};

use lazy_static::lazy_static;

const NUM_ROUNDS: usize = 81;
const ROTATION_OFFSET: usize = 364;
const STATE_SIZE: usize = HASH_LENGTH * 3;

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
    (tmp & !x_p, !tmp & !(x_p ^ y_n))
}

fn reorder(p: &mut [U256; 3], n: &mut [U256; 3]) {
    const M0: u64 = 0x9249249249249249;
    const M1: u64 = M0 << 1;
    const M2: u64 = M0 << 2;

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
