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

//! Ternary seed to derive private keys, public keys and signatures from.

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};
use bee_crypto::ternary::{
    sponge::{Kerl, Sponge},
    HASH_LENGTH,
};
use bee_ternary::{Btrit, T1B1Buf, Trit, TritBuf, Trits, TryteBuf, T1B1};

use rand::Rng;
use thiserror::Error;
use zeroize::Zeroize;

/// Errors occuring when handling a `Seed`.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Invalid seed length.
    #[error("Invalid seed length.")]
    InvalidLength(usize),
    /// Invalid seed trytes.
    #[error("Invalid seed trytes.")]
    InvalidTrytes,
    /// Failed sponge operation.
    #[error("Failed sponge operation.")]
    FailedSpongeOperation,
}

/// Ternary `Kerl`-based `Seed` to derive private keys, public keys and signatures from.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct Seed(TritBuf<T1B1Buf>);

impl Zeroize for Seed {
    fn zeroize(&mut self) {
        // This unsafe is fine since we only reset the whole buffer with zeros, there is no alignement issues.
        unsafe { self.0.as_i8_slice_mut().zeroize() }
    }
}

impl Seed {
    /// Creates a new random `Seed`.
    pub fn rand() -> Self {
        // `ThreadRng` implements `CryptoRng` so it is safe to use in cryptographic contexts.
        // https://rust-random.github.io/rand/rand/trait.CryptoRng.html
        let mut rng = rand::thread_rng();
        let trits = [Btrit::NegOne, Btrit::Zero, Btrit::PlusOne];
        let mut seed = [Btrit::Zero; HASH_LENGTH];

        for trit in seed.iter_mut() {
            *trit = trits[rng.gen_range(0, trits.len())];
        }

        Self(<&Trits>::from(&seed as &[_]).to_buf())
    }

    /// Creates a new `Seed` from the current `Seed` and an index.
    pub fn subseed(&self, index: u64) -> Self {
        let mut subseed = self.0.clone();

        for _ in 0..index {
            for t in subseed.iter_mut() {
                if let Some(ntrit) = t.checked_increment() {
                    *t = ntrit;
                    break;
                } else {
                    *t = Btrit::NegOne;
                }
            }
        }

        // Safe to unwrap since the size is known to be valid.
        Self(Kerl::default().digest(&subseed).unwrap())
    }

    /// Creates a `Seed` from &str.
    pub fn from_str(str: &str) -> Result<Self, Error> {
        if str.len() != HASH_LENGTH / 3 {
            return Err(Error::InvalidLength(str.len() * 3));
        }

        Ok(Self(
            TryteBuf::try_from_str(str)
                .map_err(|_| Error::InvalidTrytes)?
                .as_trits()
                .encode::<T1B1Buf>(),
        ))
    }

    /// Creates a `Seed` from trits.
    pub fn from_trits(buf: TritBuf<T1B1Buf>) -> Result<Self, Error> {
        if buf.len() != HASH_LENGTH {
            return Err(Error::InvalidLength(buf.len()));
        }

        Ok(Self(buf))
    }

    /// Returns the inner trits.
    pub fn as_trits(&self) -> &Trits<T1B1> {
        &self.0
    }
}
