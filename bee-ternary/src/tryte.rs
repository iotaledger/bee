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

use crate::{Btrit, Error, Trits, T3B1};
use std::{
    convert::TryFrom,
    fmt,
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

/// A ternary tryte. Equivalent to 3 trits.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(i8)]
pub enum Tryte {
    N = -13,
    O = -12,
    P = -11,
    Q = -10,
    R = -9,
    S = -8,
    T = -7,
    U = -6,
    V = -5,
    W = -4,
    X = -3,
    Y = -2,
    Z = -1,
    Nine = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
    I = 9,
    J = 10,
    K = 11,
    L = 12,
    M = 13,
}

impl Tryte {
    /// The minimum value that this [`Tryte`] can hold
    pub const MIN_VALUE: Self = Tryte::N;
    /// The maximum value that this [`Tryte`] can hold
    pub const MAX_VALUE: Self = Tryte::M;

    pub fn from_trits(trits: [Btrit; 3]) -> Self {
        let x = i8::from(trits[0]) + i8::from(trits[1]) * 3 + i8::from(trits[2]) * 9;
        Tryte::try_from(x).unwrap()
    }

    /// Interpret this tryte as a [`T3B1`] trit slice with exactly 3 elements.
    pub fn as_trits(&self) -> &Trits<T3B1> {
        unsafe { &*(T3B1::make(self as *const _ as *const _, 0, 3) as *const _) }
    }

    /// Interpret this tryte as a mutable [`T3B1`] trit slice with exactly 3 elements.
    pub fn as_trits_mut(&mut self) -> &mut Trits<T3B1> {
        unsafe { &mut *(T3B1::make(self as *const _ as *const _, 0, 3) as *mut _) }
    }
}

impl From<Tryte> for char {
    fn from(tryte: Tryte) -> char {
        match tryte as i8 {
            0 => '9',
            -13..=-1 => (((tryte as i8 + 13) as u8) + b'N') as char,
            1..=13 => (((tryte as i8 - 1) as u8) + b'A') as char,
            x => unreachable!("Tried to decode Tryte with variant {}", x),
        }
    }
}

impl fmt::Debug for Tryte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", char::from(*self))
    }
}

impl fmt::Display for Tryte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl TryFrom<char> for Tryte {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '9' => Ok(Tryte::Nine),
            'N'..='Z' => Ok(unsafe { std::mem::transmute((c as u8 - b'N') as i8 - 13) }),
            'A'..='M' => Ok(unsafe { std::mem::transmute((c as u8 - b'A') as i8 + 1) }),
            _ => Err(Error::InvalidRepr),
        }
    }
}

impl TryFrom<i8> for Tryte {
    type Error = Error;

    fn try_from(x: i8) -> Result<Self, Self::Error> {
        match x {
            -13..=13 => Ok(unsafe { std::mem::transmute(x) }),
            _ => Err(Error::InvalidRepr),
        }
    }
}

/// A buffer of [`Tryte`]s. Analagous to [`Vec`].
#[derive(Default)]
pub struct TryteBuf {
    inner: Vec<Tryte>,
}

impl TryteBuf {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new empty buffer with room for `cap` trytes.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    /// Attempt to parse a string into a tryte buffer.
    pub fn try_from_str(s: &str) -> Result<Self, Error> {
        s.chars().map(Tryte::try_from).collect()
    }

    /// Returns `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of trytes in the buffer.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Push a new tryte to the end of the buffer.
    pub fn push(&mut self, tryte: Tryte) {
        self.inner.push(tryte);
    }

    /// Attempt to pop a tryte from the end of the buffer.
    pub fn pop(&mut self) -> Option<Tryte> {
        self.inner.pop()
    }

    /// Safely interpret this tryte buffer as a [`T3B1`] trit slice.
    pub fn as_trits(&self) -> &Trits<T3B1> {
        unsafe { &*(T3B1::make(self.as_ptr() as *const _, 0, self.len() * 3) as *const _) }
    }

    /// Safely interpret this tryte buffer as a mutable [`T3B1`] trit slice.
    pub fn as_trits_mut(&mut self) -> &mut Trits<T3B1> {
        unsafe { &mut *(T3B1::make(self.as_ptr() as *const _, 0, self.len() * 3) as *mut _) }
    }
}

impl Deref for TryteBuf {
    type Target = [Tryte];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TryteBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl FromIterator<Tryte> for TryteBuf {
    fn from_iter<I: IntoIterator<Item = Tryte>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut this = Self::with_capacity(iter.size_hint().0);
        for tryte in iter {
            this.push(tryte);
        }
        this
    }
}

impl fmt::Debug for TryteBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl fmt::Display for TryteBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for tryte in self.iter() {
            write!(f, "{}", tryte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn zeroes() {
        let _ = TritBuf::<T3B1Buf>::filled(243, Btrit::Zero)
            .encode::<T3B1Buf>()
            .as_trytes()
            .iter()
            .map(|t| char::from(*t))
            .collect::<String>();
    }
}
