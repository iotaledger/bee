// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::ternary::{
    bigint::{binary_representation::U8Repr, endianness::BigEndian, error::Error as ConversionError, I384, T242, T243},
    sponge::Sponge,
    HASH_LENGTH,
};
use bee_ternary::{Btrit, Trits, T1B1};

use tiny_keccak::{Hasher, Keccak};

/// State of the ternary cryptographic function `Kerl`.
#[derive(Clone)]
pub struct Kerl {
    /// Actual keccak hash function.
    keccak: Keccak,
    /// Binary working state.
    binary_state: I384<BigEndian, U8Repr>,
    /// Ternary working state.
    ternary_state: T243<Btrit>,
}

impl Default for Kerl {
    fn default() -> Self {
        Self {
            keccak: Keccak::v384(),
            binary_state: Default::default(),
            ternary_state: Default::default(),
        }
    }
}

impl Kerl {
    /// Creates a new `Kerl`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub enum Error {
    NotMultipleOfHashLength,
    TernaryBinaryConversion(ConversionError),
}

impl From<ConversionError> for Error {
    fn from(error: ConversionError) -> Self {
        Error::TernaryBinaryConversion(error)
    }
}

impl Sponge for Kerl {
    type Error = Error;

    /// Reset the internal state by overwriting it with zeros.
    fn reset(&mut self) {
        // TODO: Overwrite the internal buffer directly rather than setting it to a new Keccak object.
        // However, `Keccak::v384::reset() is currently not exposed.
        self.keccak = Keccak::v384();
    }

    /// Absorb `input` into the sponge by copying `HASH_LENGTH` chunks of it into its internal state and transforming
    /// the state before moving on to the next chunk.
    ///
    /// If `input` is not a multiple of `HASH_LENGTH` with the last chunk having `n < HASH_LENGTH` trits, the last chunk
    /// will be copied to the first `n` slots of the internal state. The remaining data in the internal state is then
    /// just the result of the last transformation before the data was copied, and will be reused for the next
    /// transformation.
    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error> {
        if input.len() % HASH_LENGTH != 0 {
            return Err(Error::NotMultipleOfHashLength);
        }

        for trits_chunk in input.chunks(HASH_LENGTH) {
            self.ternary_state.copy_from(trits_chunk);
            // TODO: Convert to `t242` without cloning.
            self.binary_state = self.ternary_state.clone().into_t242().into();

            self.keccak.update(&self.binary_state[..]);
        }

        Ok(())
    }

    /// Squeeze the sponge by copying the calculated hash into the provided `buf`.
    /// This will fill the buffer in chunks of `HASH_LENGTH` at a time.
    ///
    /// If the last chunk is smaller than `HASH_LENGTH`, then only the fraction that fits is written into it.
    fn squeeze_into(&mut self, buf: &mut Trits<T1B1>) -> Result<(), Self::Error> {
        if buf.len() % HASH_LENGTH != 0 {
            return Err(Error::NotMultipleOfHashLength);
        }

        for trit_chunk in buf.chunks_mut(HASH_LENGTH) {
            // Create a new Keccak instead of resetting the internal one.
            let mut keccak = Keccak::v384();

            // Swap out the internal one and the new one.
            std::mem::swap(&mut self.keccak, &mut keccak);

            keccak.finalize(&mut self.binary_state[..]);
            let ternary_value = T242::from_i384_ignoring_mst(self.binary_state).into_t243();

            trit_chunk.copy_from(&ternary_value);
            self.binary_state.not_inplace();
            self.keccak.update(&self.binary_state[..]);
        }
        Ok(())
    }
}
