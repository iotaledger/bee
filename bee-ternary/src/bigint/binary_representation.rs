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

//! Binary representation of big integers.

use crate::bigint::sealed::Sealed;

/// The number of bits in an U384/I384.
pub const BINARY_LEN: usize = 384;

/// Binary representation of a big integer.
pub trait BinaryRepresentation: Sealed + Clone {
    /// Inner representation type of the big integer.
    type T;

    /// Iterates over a slice of the inner representation type of the big integer.
    fn iter(&self) -> std::slice::Iter<'_, Self::T>;
}

/// The number of u8s in an U384/I384.
pub const BINARY_LEN_IN_U8: usize = BINARY_LEN / 8;

/// The inner representation of an U384/I384 using 48 u8s.
pub type U8Repr = [u8; BINARY_LEN_IN_U8];

impl Sealed for U8Repr {}

impl BinaryRepresentation for U8Repr {
    type T = u8;

    fn iter(&self) -> std::slice::Iter<'_, Self::T> {
        (self as &[Self::T]).iter()
    }
}

/// The number of u32s in an U384/I384.
pub const BINARY_LEN_IN_U32: usize = BINARY_LEN / 32;

/// The inner representation of an U384/I384 using 12 u32s.
pub type U32Repr = [u32; BINARY_LEN_IN_U32];

impl Sealed for U32Repr {}

impl BinaryRepresentation for U32Repr {
    type T = u32;

    fn iter(&self) -> std::slice::Iter<'_, Self::T> {
        (self as &[Self::T]).iter()
    }
}
