// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::ternary::{
    sponge::curlp::batched::{
        bct::{BcTrit, BcTritArr, BcTrits},
        HIGH_BITS,
    },
    HASH_LENGTH,
};

pub(crate) struct BctCurlP {
    #[allow(deprecated)]
    rounds: crate::ternary::sponge::CurlPRounds,
    state: BcTritArr<{ 3 * HASH_LENGTH }>,
    scratch_pad: BcTritArr<{ 3 * HASH_LENGTH }>,
}

impl BctCurlP {
    #[allow(clippy::assertions_on_constants, deprecated)]
    pub(crate) fn new(rounds: crate::ternary::sponge::CurlPRounds) -> Self {
        // Ensure that changing the hash length will not cause undefined behaviour.
        assert!(3 * HASH_LENGTH > 728);
        Self {
            rounds,
            state: BcTritArr::filled(HIGH_BITS),
            scratch_pad: BcTritArr::filled(HIGH_BITS),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.state.fill(HIGH_BITS);
    }

    pub(crate) fn transform(&mut self) {
        let mut scratch_pad_index = 0;

        // All the unchecked accesses here are guaranteed to be safe by the assertion inside `new`.
        for _round in 0..self.rounds as usize {
            self.scratch_pad.copy_from_slice(&self.state);

            let BcTrit(mut alpha, mut beta) = unsafe { *self.scratch_pad.get_unchecked(scratch_pad_index) };

            scratch_pad_index += 364;

            let mut temp = unsafe { *self.scratch_pad.get_unchecked(scratch_pad_index) };

            let delta = beta ^ temp.lo();

            *unsafe { self.state.get_unchecked_mut(0) } = BcTrit(!(delta & alpha), delta | (alpha ^ temp.hi()));

            let mut state_index = 1;

            while state_index < self.state.len() {
                scratch_pad_index += 364;

                alpha = temp.lo();
                beta = temp.hi();
                temp = unsafe { *self.scratch_pad.get_unchecked(scratch_pad_index) };

                let delta = beta ^ temp.lo();

                *unsafe { self.state.get_unchecked_mut(state_index) } =
                    BcTrit(!(delta & alpha), delta | (alpha ^ temp.hi()));

                state_index += 1;

                scratch_pad_index -= 365;

                alpha = temp.lo();
                beta = temp.hi();
                temp = unsafe { *self.scratch_pad.get_unchecked(scratch_pad_index) };

                let delta = beta ^ temp.lo();

                *unsafe { self.state.get_unchecked_mut(state_index) } =
                    BcTrit(!(delta & alpha), delta | (alpha ^ temp.hi()));

                state_index += 1;
            }
        }
    }

    pub(crate) fn absorb(&mut self, bc_trits: &BcTrits) {
        let mut length = bc_trits.len();
        let mut offset = 0;

        loop {
            let length_to_copy = if length < HASH_LENGTH { length } else { HASH_LENGTH };
            // This is safe as `length_to_copy <= HASH_LENGTH`.
            unsafe { self.state.get_unchecked_mut(0..length_to_copy) }
                .copy_from_slice(unsafe { bc_trits.get_unchecked(offset..offset + length_to_copy) });

            self.transform();

            if length <= length_to_copy {
                break;
            } else {
                offset += length_to_copy;
                length -= length_to_copy;
            }
        }
    }

    // This method shouldn't assume that `result` has any particular content, just that it has an
    // adequate size.
    pub(crate) fn squeeze_into(&mut self, result: &mut BcTrits) {
        let trit_count = result.len();

        let hash_count = trit_count / HASH_LENGTH;

        for i in 0..hash_count {
            unsafe { result.get_unchecked_mut(i * HASH_LENGTH..(i + 1) * HASH_LENGTH) }
                .copy_from_slice(unsafe { self.state.get_unchecked(0..HASH_LENGTH) });

            self.transform();
        }

        let last = trit_count - hash_count * HASH_LENGTH;

        unsafe { result.get_unchecked_mut(trit_count - last..trit_count) }
            .copy_from_slice(unsafe { self.state.get_unchecked(0..last) });

        if trit_count % HASH_LENGTH != 0 {
            self.transform();
        }
    }
}
