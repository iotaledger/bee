// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::ternary::sponge::batched_curl::{bct::BCTritBuf, HIGH_BITS};

const NUMBER_OF_TRITS_IN_A_TRYTE: usize = 3;

pub struct BCTCurl {
    hash_length: usize,
    number_of_rounds: usize,
    state: BCTritBuf,
    scratch_pad: BCTritBuf,
}

impl BCTCurl {
    pub fn new(hash_length: usize, number_of_rounds: usize) -> Self {
        Self {
            hash_length,
            number_of_rounds,
            state: BCTritBuf::filled(HIGH_BITS, NUMBER_OF_TRITS_IN_A_TRYTE * hash_length),
            scratch_pad: BCTritBuf::filled(HIGH_BITS, NUMBER_OF_TRITS_IN_A_TRYTE * hash_length),
        }
    }

    pub fn reset(&mut self) {
        self.state.fill(HIGH_BITS);
    }

    pub fn transform(&mut self) {
        let mut scratch_pad_index = 0;

        for _round in 0..self.number_of_rounds {
            self.scratch_pad.as_slice_mut().copy_from_slice(self.state.as_slice());

            let mut alpha = self.scratch_pad.lo()[scratch_pad_index];
            let mut beta = self.scratch_pad.hi()[scratch_pad_index];

            for state_index in 0..self.state.len() {
                if scratch_pad_index < 365 {
                    scratch_pad_index += 364;
                } else {
                    scratch_pad_index -= 365;
                }

                let a = self.scratch_pad.lo()[scratch_pad_index];
                let b = self.scratch_pad.hi()[scratch_pad_index];

                let delta = beta ^ a;

                self.state.lo_mut()[state_index] = !(delta & alpha);
                self.state.hi_mut()[state_index] = delta | (alpha ^ b);

                alpha = a;
                beta = b;
            }
        }
    }

    pub fn absorb(&mut self, bc_trits: &BCTritBuf) {
        let mut length = bc_trits.len();
        let mut offset = 0;

        loop {
            let length_to_copy = if length < self.hash_length {
                length
            } else {
                self.hash_length
            };

            self.state
                .get_mut(0..length_to_copy)
                .copy_from_slice(bc_trits.get(offset..offset + length_to_copy));

            self.transform();

            if length <= length_to_copy {
                break;
            } else {
                offset += length_to_copy;
                length -= length_to_copy;
            }
        }
    }

    pub fn squeeze_into(&mut self, result: &mut BCTritBuf) {
        let trit_count = result.len();

        let hash_count = trit_count / self.hash_length;

        for i in 0..hash_count {
            result
                .get_mut(i * self.hash_length..(i + 1) * self.hash_length)
                .copy_from_slice(self.state.get(0..self.hash_length));

            self.transform();
        }

        let last = trit_count - hash_count * self.hash_length;

        result
            .get_mut(trit_count - last..trit_count)
            .copy_from_slice(self.state.get(0..last));

        if trit_count % self.hash_length != 0 {
            self.transform();
        }
    }
}
