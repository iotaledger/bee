// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Ternary sponge constructions.

#![allow(deprecated)]

mod curlp;
mod kerl;
mod kind;

use super::HASH_LENGTH;

pub use curlp::{BatchHasher, CurlP, CurlP27, CurlP81, CurlPRounds, UnrolledCurlP81, BATCH_SIZE};
pub use kerl::Kerl;
pub use kind::SpongeKind;

use bee_ternary::{TritBuf, Trits};

use std::ops::DerefMut;

/// The common interface of ternary cryptographic hash functions that follow the sponge construction.
#[deprecated(note = "Sponge and SpongeKind are deprecated because `iota-crypto` doesn't implement this trait.")]
pub trait Sponge {
    /// An error indicating that a failure has occured during a sponge operation.
    type Error;

    /// Reset the inner state of the sponge.
    fn reset(&mut self);

    /// Absorb `input` into the sponge.
    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error>;

    /// Squeeze the sponge into `buf`.
    fn squeeze_into(&mut self, buf: &mut Trits) -> Result<(), Self::Error>;

    /// Convenience function using `Sponge::squeeze_into` to return an owned output.
    fn squeeze(&mut self) -> Result<TritBuf, Self::Error> {
        let mut output = TritBuf::zeros(HASH_LENGTH);
        self.squeeze_into(&mut output)?;
        Ok(output)
    }

    /// Convenience function to absorb `input`, squeeze the sponge into `buf`, and reset the sponge.
    fn digest_into(&mut self, input: &Trits, buf: &mut Trits) -> Result<(), Self::Error> {
        self.absorb(input)?;
        self.squeeze_into(buf)?;
        self.reset();
        Ok(())
    }

    /// Convenience function to absorb `input`, squeeze the sponge, reset the sponge and return an owned output.
    fn digest(&mut self, input: &Trits) -> Result<TritBuf, Self::Error> {
        self.absorb(input)?;
        let output = self.squeeze()?;
        self.reset();
        Ok(output)
    }
}

impl<T: Sponge, U: DerefMut<Target = T>> Sponge for U {
    type Error = T::Error;

    fn reset(&mut self) {
        T::reset(self)
    }

    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error> {
        T::absorb(self, input)
    }

    fn squeeze_into(&mut self, buf: &mut Trits) -> Result<(), Self::Error> {
        T::squeeze_into(self, buf)
    }
}
