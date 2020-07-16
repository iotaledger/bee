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

//! This module contains unsigned integers encoded by 384 bits.

mod constants;

pub use constants::{
    BE_U32_0, BE_U32_1, BE_U32_2, BE_U32_HALF_MAX, BE_U32_HALF_MAX_T242, BE_U32_MAX, BE_U8_0, BE_U8_1, BE_U8_2,
    BE_U8_MAX, LE_U32_0, LE_U32_1, LE_U32_2, LE_U32_HALF_MAX, LE_U32_HALF_MAX_T242, LE_U32_MAX, LE_U32_MAX_T242,
    LE_U32_NEG_HALF_MAX_T242, LE_U32_ONLY_T243_OCCUPIED, LE_U8_0, LE_U8_1, LE_U8_2, LE_U8_MAX,
};

use crate::ternary::bigint::{
    binary_representation::{BinaryRepresentation, U32Repr, U8Repr},
    endianness::{BigEndian, LittleEndian},
    error::Error,
    overflowing_add::OverflowingAdd,
    split_integer::SplitInteger,
    t243, I384, T242, T243,
};

use bee_ternary::Utrit;

use byteorder::{self, ByteOrder};

use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// A big integer encoding an unsigned integer with 384 bits.
///
/// `T` is usually taken as a `[u32; 12]` or `[u8; 48]`.
///
/// `E` refers to the endianness of the digits in `T`. This means that in the case of `[u32; 12]`, if `E == BigEndian`,
/// that the u32 at position i=0 is considered the most significant digit. The endianness `E` here makes no statement
/// about the endianness of each single digit within itself (this then is dependent on the endianness of the platform
/// this code is run on).
///
/// For `E == LittleEndian` the digit at the last position is considered to be the most significant.
#[derive(Clone, Copy)]
pub struct U384<E, T> {
    pub(crate) inner: T,
    _phantom: PhantomData<E>,
}

impl<E, T> Deref for U384<E, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E, T> DerefMut for U384<E, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<E: fmt::Debug, R: BinaryRepresentation, D> fmt::Debug for U384<E, R>
where
    E: fmt::Debug,
    R: BinaryRepresentation<Inner = D>,
    D: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("U384")
            .field("inner", &self.inner.iter())
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl U384<BigEndian, U32Repr> {
    /// Reinterprets the U384 as an I384.
    pub fn as_i384(self) -> I384<BigEndian, U32Repr> {
        I384::<BigEndian, U32Repr>::from_array(self.inner)
    }

    /// Shifts the U384 in signed space.
    pub fn shift_into_i384(mut self) -> I384<BigEndian, U32Repr> {
        self.sub_inplace(*BE_U32_HALF_MAX);
        self.sub_inplace(Self::one());
        self.as_i384()
    }

    /// Adds `other` onto `self` in place.
    pub fn add_inplace(&mut self, other: Self) {
        let mut overflown = false;
        let self_iter = self.inner.iter_mut().rev();
        let other_iter = other.inner.iter().rev();

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, still_overflown) = s.overflowing_add_with_carry(*o, overflown as u32);
            *s = sum;
            overflown = still_overflown;
        }
    }

    /// Adds `other` in place, returning the number of digits required accomodate `other` (starting from the least
    /// significant one).
    pub fn add_digit_inplace<T: Into<u32>>(&mut self, other: T) -> usize {
        let other = other.into();

        let mut i = self.inner.len() - 1;

        let (sum, mut overflown) = self.inner[i].overflowing_add(other);
        self.inner[i] = sum;

        i -= 1;

        while overflown {
            let (sum, still_overflown) = self.inner[i].overflowing_add(1u32);
            self.inner[i] = sum;
            overflown = still_overflown;
            i -= 1;
        }

        i
    }

    /// Divides the u384 by 2 by bitshifting all bits one position to the right.
    pub fn divide_by_two(&mut self) {
        let mut i = self.inner.len() - 1;
        while i < self.inner.len() - 1 {
            let (left_slice, right_slice) = self.inner.split_at_mut(i + 1);
            let left = &mut left_slice[i];
            let right = &mut right_slice[0];
            *left >>= 1;
            *left |= *right << 31;
            i -= 1;
        }
        self.inner[0] >>= 1;
    }

    /// Creates an U384 from an unbalanced T242.
    pub fn from_t242(trits: T242<Utrit>) -> Self {
        let u384_le = U384::<LittleEndian, U32Repr>::from_t242(trits);
        u384_le.into()
    }

    /// Subtract `other` from `self` inplace.
    ///
    /// This function is defined in terms of `overflowing_add` by making use of the following identity (in terms of
    /// Two's complement, and where `!` is logical bitwise negation):
    ///
    /// !x = -x - 1 => -x = !x + 1
    pub fn sub_inplace(&mut self, other: Self) {
        let self_iter = self.inner.iter_mut().rev();
        let other_iter = other.inner.iter().rev();

        // The first `borrow` is always true because the addition operation needs to account for the above).
        let mut borrow = true;

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, has_overflown) = s.overflowing_add_with_carry(!*o, borrow as u32);
            *s = sum;
            borrow = has_overflown;
        }
    }

    /// Converts a signed integer represented by the balanced trits in `t243` to the unsigned binary integer `u384`.
    /// It does this by shifting the `t243` into signed range (by adding 1 to all its trits).  `t243` is assumed to be
    /// in little endian representation, with the most significant trit being at the largest index in the array.
    ///
    /// This is done in the following steps:
    ///
    /// 1. `1` is added to all balanced trits, making them *unsigned*: `{-1, 0, 1} -> {0, 1, 2}`.
    /// 2. The `t243` are converted to base 10 and through this immediately to `i384` by calculating the sum `s
    ///
    /// ```ignore
    /// s = t_242 * 3^241 + t_241 * 3^240 + ...
    ///   + t_{i+1} * 3^{i} + t_i * 3^{i-1} + t_{i-1} * 3^{i-2} + ...
    ///   + t_1 * 3 + t_0
    /// ```
    ///
    /// To perform this sum efficiently, its accumulation is staggered, so that each multiplication by 3 is done in each
    /// iteration of accumulating loop. This can be understood by factoring the powers of 3 from the previous sum:
    ///
    /// ```ignore
    /// s = (...((t_242 * 3 + t_241) * 3 + t_240) * 3 + ...
    ///   +  ...((t_{i+1} * 3 + t_i) * 3 + t_{i-1}) * 3 + ...
    ///   +  ...t_1) * 3 + t_0
    /// ```
    ///
    /// Expressed in procedural form, this is the sum accumulated in `acc` with the index `i` running from `[242..0`]:
    ///
    /// ```ignore
    /// acc = 0
    /// for i, trit in trits.rev():
    ///     acc := acc + trit * 3^i
    /// ```
    pub fn try_from_t243(trits: T243<Utrit>) -> Result<Self, Error> {
        let u384_le = U384::<LittleEndian, U32Repr>::try_from_t243(trits)?;
        Ok(u384_le.into())
    }
}

impl U384<LittleEndian, U32Repr> {
    /// Reinterprets the U384 as an I384.
    pub fn as_i384(self) -> I384<LittleEndian, U32Repr> {
        I384::<LittleEndian, U32Repr>::from_array(self.inner)
    }

    /// Adds `other` onto `self` in place.
    pub fn add_inplace(&mut self, other: Self) {
        let mut overflown = false;
        let self_iter = self.inner.iter_mut();
        let other_iter = other.inner.iter();

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, still_overflown) = s.overflowing_add_with_carry(*o, overflown as u32);
            *s = sum;
            overflown = still_overflown;
        }
    }

    /// Adds `other` in place, returning the number of digits required accomodate `other` (starting from the least
    /// significant one).
    pub fn add_digit_inplace<T: Into<u32>>(&mut self, other: T) -> usize {
        let other = other.into();

        let (sum, mut overflown) = self.inner[0].overflowing_add(other);
        self.inner[0] = sum;

        let mut i = 1;

        while overflown {
            let (sum, still_overflown) = self.inner[i].overflowing_add(1u32);
            self.inner[i] = sum;
            overflown = still_overflown;
            i += 1;
        }

        i
    }

    /// Divides the u384 by 2 by bitshifting all bits one position to the right.
    pub fn divide_by_two(&mut self) {
        let mut i = 0;
        while i < self.inner.len() - 1 {
            let (left_slice, right_slice) = self.inner.split_at_mut(i + 1);
            let left = &mut left_slice[i];
            let right = &mut right_slice[0];
            *left >>= 1;
            *left |= *right << 31;
            i += 1;
        }
        self.inner[self.inner.len() - 1] >>= 1;
    }

    /// Creates an U384 from an unbalanced T242.
    pub fn from_t242(trits: T242<Utrit>) -> Self {
        let t243 = trits.into_t243();

        // Safe, because `UT242::MAX` always fits into U384.
        Self::try_from_t243(t243).unwrap()
    }

    /// Shifts the U384 in signed space.
    pub fn shift_into_i384(mut self) -> I384<LittleEndian, U32Repr> {
        self.sub_inplace(*LE_U32_HALF_MAX);
        self.sub_inplace(Self::one());
        self.as_i384()
    }

    /// Subtract `other` from `self` inplace.
    ///
    /// This function is defined in terms of `overflowing_add` by making use of the following identity (in terms of
    /// Two's complement, and where `!` is logical bitwise negation):
    ///
    /// !x = -x -1 => -x = !x + 1
    pub fn sub_inplace(&mut self, other: Self) {
        let self_iter = self.inner.iter_mut();
        let other_iter = other.inner.iter();

        // The first `borrow` is always true because the addition operation needs to account for the above).
        let mut borrow = true;

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, has_overflown) = s.overflowing_add_with_carry(!*o, borrow as u32);
            *s = sum;
            borrow = has_overflown;
        }
    }

    /// Converts a signed integer represented by the balanced trits in `t243` to the unsigned binary integer `u384`.
    /// It does this by shifting the `t243` into signed range (by adding 1 to all its trits).  `t243` is assumed to be
    /// in little endian representation, with the most significant trit being at the largest index in the array.
    ///
    /// This is done in the following steps:
    ///
    /// 1. `1` is added to all balanced trits, making them *unsigned*: `{-1, 0, 1} -> {0, 1, 2}`.
    /// 2. The `t243` are converted to base 10 and through this immediately to `i384` by calculating the sum `s
    ///
    /// ```ignore
    /// s = t_242 * 3^241 + t_241 * 3^240 + ...
    ///   + t_{i+1} * 3^{i} + t_i * 3^{i-1} + t_{i-1} * 3^{i-2} + ...
    ///   + t_1 * 3 + t_0
    /// ```
    ///
    /// To perform this sum efficiently, its accumulation is staggered, so that each multiplication by 3 is done in each
    /// iteration of accumulating loop. This can be understood by factoring the powers of 3 from the previous sum:
    ///
    /// ```ignore
    /// s = (...((t_242 * 3 + t_241) * 3 + t_240) * 3 + ...
    ///   +  ...((t_{i+1} * 3 + t_i) * 3 + t_{i-1}) * 3 + ...
    ///   +  ...t_1) * 3 + t_0
    /// ```
    ///
    /// Expressed in procedural form, this is the sum accumulated in `acc` with the index `i` running from `[242..0`]:
    ///
    /// ```ignore
    /// acc = 0
    /// for i, trit in trits.rev():
    ///     acc := acc + trit * 3^i
    /// ```
    pub fn try_from_t243(trits: T243<Utrit>) -> Result<Self, Error> {
        if trits > *t243::UTRIT_U384_MAX {
            return Err(Error::TernaryExceedsBinaryRange);
        }

        // The accumulator is a little endian bigint using `u32` as an internal representation.
        let mut accumulator = Self::zero();
        let mut accumulator_extent = 1;

        // Iterate over all trits starting from the most significant one.
        //
        // Note that the most significant trit is that at position i=241, not i=242.
        // 384 bits cannot represent 243 trits and this we choose to ignore the technically most significant one.

        // Optimization: advance the iterator until the first non-zero trit is found.
        let mut binary_trits_iterator = trits.as_i8_slice().iter().rev().peekable();
        while let Some(0) = binary_trits_iterator.peek() {
            binary_trits_iterator.next();
        }

        for binary_trit in binary_trits_iterator {
            // Iterate over all digits in the bigint accumulator, multiplying by 3 into a `u64`.
            // Overflow is handled by taking the lower `u32` as the new digit, and the higher `u32` as the carry.
            let mut carry: u32 = 0;
            for digit in accumulator.inner[0..accumulator_extent].iter_mut() {
                let new_digit = *digit as u64 * 3u64 + carry as u64;

                *digit = new_digit.lo();
                carry = new_digit.hi();
            }

            if carry != 0 {
                unsafe {
                    *accumulator.inner.get_unchecked_mut(accumulator_extent) = carry;
                }
                accumulator_extent += 1;
            }

            let new_extent = accumulator.add_digit_inplace(*binary_trit as u32);
            if new_extent > accumulator_extent {
                accumulator_extent = new_extent;
            }
        }

        Ok(accumulator)
    }
}

impl_const_functions!(
    ( U384 ),
    { BigEndian, LittleEndian },
    { U8Repr, U32Repr }
);

impl_constants!(
    U384<BigEndian, U8Repr> => [
        (zero, BE_U8_0),
        (one, BE_U8_1),
        (two, BE_U8_2),
        (max, BE_U8_MAX),
    ],
    U384<LittleEndian, U8Repr> => [
        (zero, LE_U8_0),
        (one, LE_U8_1),
        (two, LE_U8_2),
        (max, LE_U8_MAX),
    ],
    U384<BigEndian, U32Repr> => [
        (zero, BE_U32_0),
        (one, BE_U32_1),
        (two, BE_U32_2),
        (max, BE_U32_MAX),
    ],
    U384<LittleEndian, U32Repr> => [
        (zero, LE_U32_0),
        (one, LE_U32_1),
        (two, LE_U32_2),
        (max, LE_U32_MAX),
    ],
);

impl From<U384<BigEndian, U32Repr>> for U384<BigEndian, U8Repr> {
    fn from(value: U384<BigEndian, U32Repr>) -> Self {
        let mut u384_u8 = Self::zero();
        byteorder::BigEndian::write_u32_into(&value.inner, &mut u384_u8.inner);
        u384_u8
    }
}

impl From<U384<LittleEndian, U8Repr>> for U384<LittleEndian, U32Repr> {
    fn from(value: U384<LittleEndian, U8Repr>) -> Self {
        let mut u384_u32 = U384::<LittleEndian, U32Repr>::zero();
        byteorder::LittleEndian::read_u32_into(&value.inner, &mut u384_u32.inner);
        u384_u32
    }
}

impl From<T242<Utrit>> for U384<LittleEndian, U32Repr> {
    fn from(value: T242<Utrit>) -> Self {
        Self::from_t242(value)
    }
}

impl Eq for U384<LittleEndian, U32Repr> {}

impl PartialEq for U384<LittleEndian, U32Repr> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd for U384<LittleEndian, U32Repr> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;

        let self_iter = self.inner.iter().rev();
        let other_iter = other.inner.iter().rev();

        let zipped_iter = self_iter.zip(other_iter);

        for (s, o) in zipped_iter {
            if s > o {
                return Some(Greater);
            } else if s < o {
                return Some(Less);
            }
        }

        Some(Equal)
    }
}

impl Ord for U384<LittleEndian, U32Repr> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,

            // The ordering is total, hence `partial_cmp` will never return `None`.
            None => unreachable!(),
        }
    }
}

impl TryFrom<T243<Utrit>> for U384<LittleEndian, U32Repr> {
    type Error = Error;

    fn try_from(value: T243<Utrit>) -> Result<Self, Self::Error> {
        Self::try_from_t243(value)
    }
}

impl_toggle_endianness!((U384), U8Repr, U32Repr);
