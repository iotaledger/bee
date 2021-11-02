// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod transform;
mod u256;

use u256::U256;

use super::{Sponge, HASH_LENGTH};

use bee_ternary::{Btrit, Trits};

use std::convert::Infallible;

enum SpongeDirection {
    Absorb,
    Squeeze,
}

/// Unrolled [`CurlP`] with a fixed number of 81 rounds.
pub struct UnrolledCurlP81 {
    p: [U256; 3],
    n: [U256; 3],
    direction: SpongeDirection,
}

impl UnrolledCurlP81 {
    /// Creates a new [`UnrolledCurlP81`].
    pub fn new() -> Self {
        Self::default()
    }

    fn squeeze_aux(&mut self, hash: &mut Trits) {
        if let SpongeDirection::Squeeze = self.direction {
            self.transform();
        }

        self.direction = SpongeDirection::Squeeze;

        for i in 0..HASH_LENGTH {
            // SAFETY: `U256::bit` returns an `i8` between `0` and `1`.
            // Substracting two bits will produce an `i8` between `-1` and `1` and matches the `repr` of `Btrit`.
            let trit = unsafe { std::mem::transmute::<i8, Btrit>(self.p[0].bit(i) - self.n[0].bit(i)) };
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

    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error> {
        assert!(!(input.is_empty() || input.len() % HASH_LENGTH != 0), "trits slice length must be multiple of {}", HASH_LENGTH);

        if let SpongeDirection::Squeeze = self.direction {
            panic!("absorb after squeeze");
        }

        for chunk in input.chunks(HASH_LENGTH) {
            let mut p = U256::default();
            let mut n = U256::default();

            for (i, trit) in chunk.iter().enumerate() {
                match trit {
                    Btrit::PlusOne => p.set_bit(i),
                    Btrit::Zero => (),
                    Btrit::NegOne => n.set_bit(i),
                }
            }

            self.p[0] = p;
            self.n[0] = n;
            self.transform();
        }

        Ok(())
    }

    fn squeeze_into(&mut self, buf: &mut Trits) -> Result<(), Self::Error> {
        assert_eq!(buf.len() % HASH_LENGTH, 0, "Invalid squeeze length");

        for chunk in buf.chunks_mut(HASH_LENGTH) {
            self.squeeze_aux(chunk);
        }

        Ok(())
    }
}
