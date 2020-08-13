use crate::ternary::sponge::batched_curl::{mux::BCTritBuf, HIGH_BITS};

const NUMBER_OF_TRITS_IN_A_TRYTE: usize = 3;

pub struct BCTCurl {
    hash_length: usize,
    number_of_rounds: usize,
    state: BCTritBuf,
}

impl BCTCurl {
    pub fn new(hash_length: usize, number_of_rounds: usize) -> Self {
        Self {
            hash_length,
            number_of_rounds,
            state: BCTritBuf::filled(HIGH_BITS, NUMBER_OF_TRITS_IN_A_TRYTE * hash_length),
        }
    }

    pub fn reset(&mut self) {
        for i in 0..self.state.len() {
            self.state.lo_mut()[i] = HIGH_BITS;
            self.state.hi_mut()[i] = HIGH_BITS;
        }
    }

    pub fn transform(&mut self) {
        let mut scratch_pad;
        let mut scratch_pad_index = 0;

        for _round in 0..self.number_of_rounds {
            scratch_pad = self.state.clone();

            let mut alpha = unsafe { *scratch_pad.lo().get_unchecked(scratch_pad_index) };
            let mut beta = unsafe { *scratch_pad.hi().get_unchecked(scratch_pad_index) };

            for state_index in 0..self.state.len() {
                if scratch_pad_index < 365 {
                    scratch_pad_index += 364;
                } else {
                    scratch_pad_index -= 365;
                }

                let a = unsafe { *scratch_pad.lo().get_unchecked(scratch_pad_index) };
                let b = unsafe { *scratch_pad.hi().get_unchecked(scratch_pad_index) };

                let delta = beta ^ a;

                *unsafe { self.state.lo_mut().get_unchecked_mut(state_index) } = !(delta & alpha);
                *unsafe { self.state.hi_mut().get_unchecked_mut(state_index) } = delta | (alpha ^ b);

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
