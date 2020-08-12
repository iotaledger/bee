use crate::ternary::sponge::batched_curl::mux::BCTrits;

const NUMBER_OF_TRITS_IN_A_TRYTE: usize = 3;

pub struct BCTCurl {
    hash_length: usize,
    number_of_rounds: usize,
    high_long_bits: usize,
    state_length: usize,
    state: BCTrits,
}

impl BCTCurl {
    pub fn new(hash_length: usize, number_of_rounds: usize, batch_size: usize) -> Self {
        let mut high_long_bits = 0;

        for i in 0..batch_size {
            high_long_bits += 1 << i;
        }

        let mut this = Self {
            hash_length,
            number_of_rounds,
            high_long_bits,
            state_length: NUMBER_OF_TRITS_IN_A_TRYTE * hash_length,
            state: BCTrits {
                lo: vec![0; NUMBER_OF_TRITS_IN_A_TRYTE * hash_length],
                hi: vec![0; NUMBER_OF_TRITS_IN_A_TRYTE * hash_length],
            },
        };

        this.reset();

        this
    }

    pub fn reset(&mut self) {
        for i in 0..self.state_length {
            self.state.lo[i] = self.high_long_bits;
            self.state.hi[i] = self.high_long_bits;
        }
    }

    pub fn transform(&mut self) {
        let mut scratch_pad_lo;
        let mut scratch_pad_hi;
        let mut scratch_pad_index = 0;

        for _round in (1..=self.number_of_rounds).rev() {
            scratch_pad_lo = self.state.lo.clone();
            scratch_pad_hi = self.state.hi.clone();

            for state_index in 0..self.state_length {
                let alpha = scratch_pad_lo[scratch_pad_index];
                let beta = scratch_pad_hi[scratch_pad_index];

                if scratch_pad_index < 365 {
                    scratch_pad_index += 364;
                } else {
                    scratch_pad_index -= 365;
                }

                let delta = beta ^ scratch_pad_lo[scratch_pad_index];

                self.state.lo[state_index] = !(delta & alpha);
                self.state.hi[state_index] = delta | (alpha ^ scratch_pad_hi[scratch_pad_index]);
            }
        }
    }

    pub fn absorb(&mut self, bc_trits: BCTrits) {
        let mut length = bc_trits.lo.len();
        let mut offset = 0;

        loop {
            let length_to_copy = if length < self.hash_length {
                length
            } else {
                self.hash_length
            };

            for i in 0..length_to_copy {
                self.state.lo[i] = bc_trits.lo[offset + i];
                self.state.hi[i] = bc_trits.hi[offset + i];
            }

            self.transform();


            if length <= length_to_copy {
                break;
            } else {
                offset += length_to_copy;
                length -= length_to_copy;
            }
        }
    }

    pub fn squeeze(&mut self, trit_count: usize) -> BCTrits {
        let mut result = BCTrits {
            lo: vec![0; trit_count],
            hi: vec![0; trit_count],
        };

        let hash_count = trit_count / self.hash_length;

        for i in 0..hash_count {
            for j in 0..self.hash_length {
                result.lo[i * self.hash_length + j] = self.state.lo[j];
                result.hi[i * self.hash_length + j] = self.state.hi[j];
            }

            self.transform();
        }

        let last = trit_count - hash_count * self.hash_length;

        for i in 0..last {
            result.lo[trit_count - last + i] = self.state.lo[i];
            result.hi[trit_count - last + i] = self.state.hi[i];
        }

        if trit_count % self.hash_length != 0 {
            self.transform();
        }

        result
    }
}
