/*
 * Copyright (c) 2019 c-mnd
 *
 * MIT License
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#![allow(dead_code)]

use super::constants::{
    Trit,
    COLUMNS,
    FROUND_CONSTANTS,
    NUM_ROUNDS,
    ROWS,
    SLICES,
    SLICESIZE,
    TROIKA_RATE,
};
use crate::Result;
use core::fmt;

#[derive(Clone, Copy)]
struct T27 {
    pub p: u32,
    pub n: u32,
}

impl fmt::Debug for T27 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T27: [p: {}, n: {}]", self.p, self.n,)
    }
}

impl T27 {
    fn new(p: u32, n: u32) -> T27 {
        T27 { p: p, n: n }
    }
    //#[inline]
    fn clean(&self) -> T27 {
        T27::new(self.p & 0x07ffffffu32, self.n & 0x07ffffffu32)
    }
    //#[inline]
    fn add(&self, other: &T27) -> T27 {
        let self_zero: u32 = !self.p & !self.n;
        let p = !(self.n ^ other.n) & !(self_zero ^ other.p);
        let n = !(self.p ^ other.p) & !(self_zero ^ other.n);
        T27::new(p, n)
    }
    //#[inline]
    fn mul(&self, other: &T27) -> T27 {
        let p = (self.p & other.p) | (self.n & other.n);
        let n = (self.p & other.n) | (self.n & other.p);
        T27::new(p, n)
    }
    //#[inline]
    fn zero() -> T27 {
        T27::new(0, 0)
    }
    //#[inline]
    fn one() -> T27 {
        T27::new(0x07ffffffu32, 0)
    }
    //#[inline]
    fn minus() -> T27 {
        T27::new(0, 0x07ffffffu32)
    }
    //#[inline]
    fn dec(&self) -> T27 {
        T27::minus().add(&self)
    }
    //#[inline]
    fn inc(&self) -> T27 {
        T27::one().add(&self)
    }
    //#[inline]
    fn set(&mut self, pos: usize, value: Trit) {
        let mask: u32 = 1u32 << pos;
        //self.p &= !mask;
        //self.n &= !mask;
        match value {
            1 => self.p |= mask,
            2 => self.n |= mask,
            _ => (),
        }
    }
    //#[inline]
    pub fn get(&mut self, pos: usize) -> Trit {
        let mask: u32 = 1u32 << pos;
        if self.p & mask != 0 {
            return 1;
        } else if self.n & mask != 0 {
            return 2;
        }
        0
    }
    //#[inline]
    fn roll(&self, by: usize) -> T27 {
        let p = ((self.p << by) | (self.p >> (27 - by))) & 0x07ffffff;
        let n = ((self.n << by) | (self.n >> (27 - by))) & 0x07ffffff;
        T27::new(p, n)
    }
}

/// The Ftroika struct is a Sponge that uses the Troika
/// hashing algorithm.
/// ```rust
/// extern crate troika_rust;
/// use troika_rust::Ftroika;
/// // Create an array of 243 1s
/// let input = [1; 243];
/// // Create an array of 243 0s
/// let mut out = [0; 243];
/// let mut ftroika = Ftroika::default();
/// ftroika.absorb(&input);
/// ftroika.finalize();
/// ftroika.squeeze(&mut out);
/// ```
#[derive(Clone, Copy)]
pub struct Ftroika {
    num_rounds: usize,
    idx: usize,
    rowcol: usize,
    slice: usize,
    state: [T27; SLICESIZE],
}

impl Default for Ftroika {
    fn default() -> Ftroika {
        Ftroika {
            num_rounds: NUM_ROUNDS,
            idx: 0,
            rowcol: 0,
            slice: 0,
            state: [T27::zero(); SLICESIZE],
        }
    }
}

impl fmt::Debug for Ftroika {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ftroika: [rounds: [{}], state: {:?}", self.num_rounds, self.state,)
    }
}

impl Ftroika {
    //#[inline]
    pub fn new(num_rounds: usize) -> Result<Ftroika> {
        let mut troika = Ftroika::default();
        troika.num_rounds = num_rounds;
        Ok(troika)
    }

    fn state(&self) -> &[T27] {
        &self.state
    }

    //#[inline]
    pub fn reset(&mut self) {
        self.state = [T27::zero(); SLICESIZE];
        self.reset_counters();
    }
    //#[inline]
    fn reset_counters(&mut self) {
        self.idx = 0;
        self.rowcol = 0;
        self.slice = 0;
    }
    //#[inline]
    fn set(&mut self, trit: Trit) {
        self.state[self.rowcol].set(self.slice, trit);
    }
    //#[inline]
    fn get(&mut self) -> Trit {
        self.state[self.rowcol].get(self.slice)
    }
    //#[inline]
    fn nullify_rate(&mut self) {
        let mask = 0x07fffe00u32;
        for i in 0..SLICESIZE {
            self.state[i].p &= mask;
            self.state[i].n &= mask;
        }
    }

    pub fn absorb(&mut self, trits: &[Trit]) {
        let mut length = trits.len();
        let mut space;
        let mut trit_idx = 0;
        while length > 0 {
            if self.idx == 0 {
                self.nullify_rate();
            }
            space = TROIKA_RATE - self.idx;
            if length < space {
                space = length;
            }
            for _ in 0..space {
                self.set(trits[trit_idx]);
                self.idx += 1;
                self.rowcol += 1;
                trit_idx += 1;
                if self.rowcol == SLICESIZE {
                    self.rowcol = 0;
                    self.slice += 1;
                }
            }
            length -= space;
            if self.idx == TROIKA_RATE {
                self.permutation();
                self.idx = 0;
                self.rowcol = 0;
                self.slice = 0;
            }
        }
    }
    pub fn finalize(&mut self) {
        let pad: [Trit; 1] = [1];
        self.absorb(&pad);
        if self.idx != 0 {
            self.permutation();
            self.reset_counters();
        }
    }

    pub fn squeeze(&mut self, trits: &mut [Trit]) {
        let mut length = trits.len();
        let mut space;
        let mut trit_idx = 0;
        while length > 0 {
            space = TROIKA_RATE - self.idx;
            if length < space {
                space = length;
            }
            for _ in 0..space {
                trits[trit_idx] = self.get();
                self.idx += 1;
                self.rowcol += 1;
                trit_idx += 1;
                if self.rowcol == SLICESIZE {
                    self.rowcol = 0;
                    self.slice += 1;
                }
            }
            //trit_idx += space;
            length -= space;
            if self.idx == TROIKA_RATE {
                self.permutation();
                self.reset_counters();
            }
        }
    }

    //#[inline]
    fn permutation(&mut self) {
        assert!(self.num_rounds <= NUM_ROUNDS);

        for round in 0..self.num_rounds {
            self.sub_trytes();
            self.shift_rows();
            self.shift_lanes();
            self.add_column_parity();
            self.add_round_constant(round);
        }
    }
    //#[inline]
    fn sub_tryte(a: &mut [T27]) {
        let d = a[0].dec();
        let e = d.mul(&a[1]).add(&a[2]);
        let f = e.mul(&a[1]).add(&d);
        let g = e.mul(&f).add(&a[1]);
        a[2] = e.clean();
        a[1] = f.clean();
        a[0] = g.clean();
    }
    //#[inline]
    fn sub_trytes(&mut self) {
        for rowcol in (0..SLICESIZE).step_by(3) {
            Ftroika::sub_tryte(&mut self.state[rowcol..(rowcol + 3)]);
        }
    }
    //#[inline]
    fn shift_rows(&mut self) {
        const SHIFTS: [u8; 27] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 9, 10, 11, 24, 25, 26, 18,
            19, 20, 21, 22, 23,
        ];
        let mut new_state = [T27::zero(); SLICESIZE];
        for i in 0..SLICESIZE {
            new_state[SHIFTS[i] as usize] = self.state[i];
        }
        self.state = new_state;
    }
    //#[inline]
    fn shift_lanes(&mut self) {
        const SHIFTS: [u8; 27] = [
            19, 13, 21, 10, 24, 15, 2, 9, 3, 14, 0, 6, 5, 1, 25, 22, 23, 20, 7, 17, 26,
            12, 8, 18, 16, 11, 4,
        ];
        let mut new_state = [T27::zero(); SLICESIZE];
        for i in 0..SLICESIZE {
            new_state[i as usize] = self.state[i].roll(SHIFTS[i] as usize);
        }
        self.state = new_state;
    }

    //#[inline]
    fn add_column_parity(&mut self) {
        let mut parity = [T27::zero(); COLUMNS];
        for col in 0..COLUMNS {
            let mut col_sum = T27::zero();
            for row in 0..ROWS {
                col_sum = col_sum.add(&self.state[COLUMNS * row + col]);
            }
            parity[col] = col_sum;
        }
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let idx = COLUMNS * row + col;
                let t1 = parity[if col == 0 { COLUMNS - 1 } else { col - 1 }];
                let t2 =
                    parity[if col == COLUMNS - 1 { 0 } else { col + 1 }].roll(SLICES - 1);
                let sum_to_add = t1.add(&t2);
                self.state[idx] = self.state[idx].add(&sum_to_add);
            }
        }
    }

    //#[inline]
    fn add_round_constant(&mut self, round: usize) {
        for col in 0..COLUMNS {
            let round_const = T27::new(
                FROUND_CONSTANTS[round][col][0],
                FROUND_CONSTANTS[round][col][1],
            );
            //let bla = self.state[col];
            self.state[col] = self.state[col].add(&round_const);
        }
    }
}

#[cfg(test)]
mod test_ftroika {
    use super::*;

    const HASH: [u8; 243] = [
        0, 2, 2, 1, 2, 1, 0, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 2, 1, 2,
        1, 2, 2, 1, 1, 1, 0, 1, 0, 2, 1, 0, 0, 0, 1, 2, 0, 2, 1, 0, 0, 2, 1, 1, 1, 1, 1,
        2, 0, 1, 0, 2, 1, 1, 2, 0, 1, 1, 1, 1, 1, 2, 2, 0, 0, 2, 2, 2, 2, 0, 0, 2, 2, 2,
        1, 2, 2, 0, 2, 1, 1, 2, 1, 1, 1, 2, 2, 1, 1, 0, 0, 0, 2, 2, 2, 0, 2, 1, 1, 1, 1,
        0, 0, 1, 0, 2, 0, 2, 0, 2, 0, 0, 0, 0, 1, 1, 1, 0, 2, 1, 1, 1, 0, 2, 0, 0, 1, 0,
        1, 0, 2, 0, 2, 2, 0, 0, 2, 2, 0, 1, 2, 1, 0, 0, 1, 2, 1, 1, 0, 0, 1, 1, 0, 2, 1,
        1, 0, 1, 2, 0, 0, 0, 1, 2, 2, 1, 1, 1, 0, 0, 2, 0, 1, 1, 2, 1, 1, 2, 1, 0, 1, 2,
        2, 2, 2, 1, 2, 0, 2, 2, 1, 2, 1, 2, 1, 2, 2, 1, 1, 2, 0, 2, 1, 0, 1, 1, 1, 0, 2,
        2, 0, 0, 2, 0, 2, 0, 1, 2, 0, 0, 2, 2, 1, 1, 2, 0, 1, 0, 0, 0, 0, 2, 0, 2, 2, 2,
    ];

    #[test]
    fn test_hash() {
        let mut ftroika = Ftroika::default();
        let mut output = [0u8; 243];
        let input = [0u8; 243];
        ftroika.absorb(&input);
        ftroika.finalize();
        ftroika.squeeze(&mut output);

        assert!(
            output.iter().zip(HASH.iter()).all(|(a, b)| a == b),
            "Arrays are not equal"
        );
    }
}
