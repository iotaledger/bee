//! Troika implementation.

use crate::constants::{
    COLUMNS,
    NUM_MAX_ROUNDS,
    NUM_SBOXES,
    ROWS,
    SLICES,
    SLICE_SIZE,
    STATE_SIZE,
    TROIKA_RATE,
};
use crate::types::Trit;

use super::constants::PADDING;
use super::luts::{ROUND_CONSTANTS, SBOX_LOOKUP, SHIFT_ROWS_LANES};

use core::fmt;

/// The Troika struct is a Sponge that uses the Troika
/// hashing algorithm.
///
/// ```
/// use bee_troika::Troika;
///
/// // Create an array of 243 1s
/// let input = [1; 243];
///
/// // Create an array of 243 0s
/// let mut out = [0; 243];
/// let mut troika = Troika::default();
///
/// troika.absorb(&input);
/// troika.squeeze(&mut out);
/// ```
#[derive(Clone, Copy)]
pub struct Troika {
    num_rounds: usize,
    state: [Trit; STATE_SIZE],
}

impl Default for Troika {
    fn default() -> Troika {
        Troika { num_rounds: NUM_MAX_ROUNDS, state: [0u8; STATE_SIZE] }
    }
}

impl fmt::Debug for Troika {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Troika: [rounds: [{}], state: {:?}", self.num_rounds, &self.state[..],)
    }
}

impl Troika {
    /// Creates a new Troika instance.
    ///
    /// # Example
    /// ```
    /// use bee_troika::Troika;
    ///
    /// let troika = Troika::new(12);
    /// ```  
    pub fn new(num_rounds: usize) -> Self {
        Self { num_rounds, ..Default::default() }
    }

    /// Returns the current state.
    pub fn state(&self) -> &[Trit] {
        &self.state
    }

    /// Resets the current state.
    pub fn reset(&mut self) {
        self.state = [0; STATE_SIZE];
    }

    /// Absorbs a slice of trits.
    pub fn absorb(&mut self, message: &[Trit]) {
        let mut message_length = message.len();
        let mut message_idx = 0;
        let mut trit_idx = 0;

        while message_length >= TROIKA_RATE {
            // Copy message block over the state
            for trit_idx in 0..TROIKA_RATE {
                self.state[trit_idx] = message[message_idx + trit_idx];
            }
            self.permutation();
            message_length -= TROIKA_RATE;
            message_idx += TROIKA_RATE;
        }

        // Pad last block
        let mut last_block = [0u8; TROIKA_RATE];

        // Copy over last incomplete message block
        for _ in 0..message_length {
            last_block[trit_idx] = message[trit_idx];
            trit_idx += 1;
        }

        // TODO: Check trit_idx is right here
        // Apply padding
        last_block[trit_idx] = PADDING;

        // Insert last message block
        for trit_idx in 0..TROIKA_RATE {
            self.state[trit_idx] = last_block[trit_idx];
        }
    }

    /// Determines the hash of the current state.
    pub fn squeeze(&mut self, hash: &mut [Trit]) {
        let mut hash_length = hash.len();
        let mut hash_idx = 0;

        while hash_length >= TROIKA_RATE {
            self.permutation();
            // Extract rate output
            for trit_idx in 0..TROIKA_RATE {
                hash[hash_idx + trit_idx] = self.state[trit_idx];
            }
            hash_idx += TROIKA_RATE;
            hash_length -= TROIKA_RATE;
        }

        // Check if there is a last incomplete block
        if hash_length % TROIKA_RATE != 0 {
            self.permutation();
            for trit_idx in 0..hash_length {
                hash[trit_idx] = self.state[trit_idx];
            }
        }
    }

    fn permutation(&mut self) {
        assert!(self.num_rounds <= NUM_MAX_ROUNDS);

        for round in 0..self.num_rounds {
            self.sub_trytes();
            self.shift_rows_lanes();
            self.add_column_parity();
            self.add_round_constant(round);
        }
    }

    fn sub_trytes(&mut self) {
        for sbox_idx in 0..NUM_SBOXES {
            let sbox_input = 9 * self.state[3 * sbox_idx]
                + 3 * self.state[3 * sbox_idx + 1]
                + self.state[3 * sbox_idx + 2];
            let mut sbox_output = SBOX_LOOKUP[sbox_input as usize];
            self.state[3 * sbox_idx + 2] = sbox_output % 3;
            sbox_output /= 3;
            self.state[3 * sbox_idx + 1] = sbox_output % 3;
            sbox_output /= 3;
            self.state[3 * sbox_idx] = sbox_output % 3;
        }
    }

    fn shift_rows_lanes(&mut self) {
        let mut new_state = [0u8; STATE_SIZE];
        for i in 0..STATE_SIZE {
            new_state[i] = self.state[SHIFT_ROWS_LANES[i]];
        }

        self.state = new_state;
    }

    fn add_column_parity(&mut self) {
        let mut parity = [0u8; SLICES * COLUMNS];

        // First compute parity for each column
        for slice in 0..SLICES {
            for col in 0..COLUMNS {
                let mut col_sum = 0;
                for row in 0..ROWS {
                    col_sum += self.state[SLICE_SIZE * slice + COLUMNS * row + col];
                }
                parity[COLUMNS * slice + col] = col_sum % 3;
            }
        }

        // Add parity
        for slice in 0..SLICES {
            for row in 0..ROWS {
                for col in 0..COLUMNS {
                    let idx = SLICE_SIZE * slice + COLUMNS * row + col;
                    let sum_to_add = parity[(col + 8) % 9 + COLUMNS * slice]
                        + parity[(col + 1) % 9 + COLUMNS * ((slice + 1) % SLICES)];
                    self.state[idx] = (self.state[idx] + sum_to_add) % 3;
                }
            }
        }
    }

    fn add_round_constant(&mut self, round: usize) {
        for slice in 0..SLICES {
            for col in 0..COLUMNS {
                let idx = SLICE_SIZE * slice + col;
                self.state[idx] =
                    (self.state[idx] + ROUND_CONSTANTS[round][slice * COLUMNS + col]) % 3;
            }
        }
    }
}

#[cfg(test)]
mod test_troika {
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
    fn troika_all_zeros_as_input() {
        let mut troika = Troika::default();
        let mut output = [0u8; 243];
        let input = [0u8; 243];
        troika.absorb(&input);
        troika.squeeze(&mut output);

        assert!(
            output.iter().zip(HASH.iter()).all(|(a, b)| a == b),
            "Arrays are not equal"
        );
    }
}
