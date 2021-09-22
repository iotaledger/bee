// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use u256::U256;

use super::{Sponge, HASH_LENGTH};

use bee_ternary::{Btrit, Trits};

use std::convert::Infallible;

mod transform;
mod u256;

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
        Self::default()
    }

    fn squeeze_aux(&mut self, mut hash: &mut Trits) {
        if let SpongeDirection::Squeeze = self.direction {
            self.transform();
        }

        self.direction = SpongeDirection::Squeeze;

        hash = &mut hash[..HASH_LENGTH];

        for i in 0..HASH_LENGTH {
            let trit = match (self.p[0].bit(i), self.n[0].bit(i)) {
                (a, b) if a > b => Btrit::PlusOne,
                (a, b) if a < b => Btrit::NegOne,
                _ => Btrit::Zero,
            };

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
        assert_eq!(buf.len() % HASH_LENGTH, 0, "Invalid squeeze length");

        while {
            self.squeeze_aux(buf);
            buf = &mut buf[HASH_LENGTH..];
            buf.len() >= HASH_LENGTH
        } {}

        Ok(())
    }
}
