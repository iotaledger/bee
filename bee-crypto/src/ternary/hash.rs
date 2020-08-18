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

use bee_ternary::{Btrit, Trits, T1B1};

use std::{
    cmp::PartialEq,
    convert::TryFrom,
    fmt, hash,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub enum Error {
    WrongLength,
}

/// The length of a hash in units of balanced trits.
pub const HASH_LENGTH: usize = 243;

/// Ternary cryptographic hash.
#[derive(Copy, Clone)]
pub struct Hash([Btrit; HASH_LENGTH]);

impl Hash {
    /// Creates a hash filled with zeros.
    pub fn zeros() -> Self {
        Self([Btrit::Zero; HASH_LENGTH])
    }

    /// Interpret the `Hash` as a trit slice.
    pub fn as_trits(&self) -> &Trits<T1B1> {
        &*self
    }

    /// Interpret the `Hash` as a mutable trit slice.
    pub fn as_trits_mut(&mut self) -> &mut Trits<T1B1> {
        &mut *self
    }

    /// Returns the weight - number of ending 0s - of the `Hash`.
    #[allow(clippy::cast_possible_truncation)] // `HASH_LENGTH` is smaller than `u8::MAX`.
    pub fn weight(&self) -> u8 {
        self.iter().rev().take_while(|t| *t == Btrit::Zero).count() as u8
    }
}

impl<'a> TryFrom<&'a Trits> for Hash {
    type Error = Error;

    fn try_from(trits: &'a Trits) -> Result<Self, Self::Error> {
        if trits.len() == HASH_LENGTH {
            let mut hash = Self([Btrit::Zero; HASH_LENGTH]);
            hash.copy_from(trits);
            Ok(hash)
        } else {
            Err(Error::WrongLength)
        }
    }
}

impl Deref for Hash {
    type Target = Trits<T1B1>;

    fn deref(&self) -> &Trits<T1B1> {
        <&Trits>::from(&self.0 as &[_])
    }
}

impl DerefMut for Hash {
    fn deref_mut(&mut self) -> &mut Trits<T1B1> {
        <&mut Trits>::from(&mut self.0 as &mut [_])
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.as_trits() == other.as_trits()
    }
}

impl Eq for Hash {}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_trits())
    }
}

impl hash::Hash for Hash {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher)
    }
}
