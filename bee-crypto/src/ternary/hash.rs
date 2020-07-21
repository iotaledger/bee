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

use bee_ternary::{Trits, T1B1};

use std::{cmp::PartialEq, fmt, hash};

/// The length of a hash in units of balanced trits.
pub const HASH_LENGTH: usize = 243;

/// Ternary cryptographic hash.
#[derive(Copy, Clone)]
pub struct Hash([i8; HASH_LENGTH]);

impl Hash {
    /// Creates a new `Hash` from trits.
    pub fn new(trits: [i8; HASH_LENGTH]) -> Self {
        Self(trits)
    }

    /// Creates a hash filled with zeros.
    pub fn zeros() -> Self {
        Self([0; HASH_LENGTH])
    }

    /// Reinterprets the `Hash` as `Trits`.
    pub fn as_trits(&self) -> &Trits<T1B1> {
        unsafe { Trits::from_raw_unchecked(&self.0, HASH_LENGTH) }
    }

    /// Returns the weight - number of ending 0s - of the `Hash`.
    pub fn weight(&self) -> u8 {
        let mut weight = 0u8;

        for i in (0..self.0.len()).rev() {
            if self.0[i] != 0 {
                break;
            }
            weight += 1;
        }

        weight
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for Hash {}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_trits())
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
