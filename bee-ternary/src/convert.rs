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

use crate::{Btrit, RawEncoding, RawEncodingBuf, Trit, TritBuf, Trits, Utrit};
use num_traits::{AsPrimitive, CheckedAdd, CheckedSub, FromPrimitive, Num, Signed};
use std::{cmp::PartialOrd, convert::TryFrom};

/// An error that may be produced during numeric conversion.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The trit slice didn't contain enough trits to be considered a numeric value.
    Empty,
    /// An overflow occurred during a numeric operation.
    Overflow,
    /// An underflow occurred during a numeric operation.
    Underflow,
}

// TODO: Find a way to implement this without conflicting impls
// impl<I, T: RawEncodingBuf> From<I> for TritBuf<T>
// where
//     T::Slice: RawEncoding<Trit = Btrit>,
//     I: AsPrimitive<i8> + FromPrimitive + Signed,
// {
//     fn from(x: I) -> Self {
//         signed_int_trits(x).collect()
//     }
// }

macro_rules! signed_try_from_trits {
    ($int:ident) => {
        impl<'a, T: RawEncoding<Trit = Btrit> + ?Sized> TryFrom<&'a Trits<T>> for $int {
            type Error = Error;
            fn try_from(trits: &'a Trits<T>) -> Result<Self, Self::Error> {
                trits_to_int(trits)
            }
        }

        impl<T: RawEncodingBuf> From<$int> for TritBuf<T>
        where
            T::Slice: RawEncoding<Trit = Btrit>,
        {
            fn from(x: $int) -> Self {
                signed_int_trits(x).collect()
            }
        }
    };
}

// We have to implement manually due to Rust's orphan rules :(
// This macro accepts anything that implements:
// `Clone + CheckedAdd + Signed + AsPrimitive<i8> + FromPrimitive`
#[cfg(has_i128)]
signed_try_from_trits!(i128);
signed_try_from_trits!(i64);
signed_try_from_trits!(i32);
signed_try_from_trits!(i16);
signed_try_from_trits!(i8);

macro_rules! unsigned_try_from_trits {
    ($int:ident) => {
        impl<'a, T: RawEncoding<Trit = Utrit> + ?Sized> TryFrom<&'a Trits<T>> for $int {
            type Error = Error;
            fn try_from(trits: &'a Trits<T>) -> Result<Self, Self::Error> {
                trits_to_int(trits)
            }
        }

        impl<T: RawEncodingBuf> From<$int> for TritBuf<T>
        where
            T::Slice: RawEncoding<Trit = Utrit>,
        {
            fn from(x: $int) -> Self {
                unsigned_int_trits(x).collect()
            }
        }
    };
}

// We have to implement manually due to Rust's orphan rules :(
// This macro accepts anything that implements:
// `Clone + CheckedAdd + Unsigned + AsPrimitive<u8> + FromPrimitive`
#[cfg(has_i128)]
unsigned_try_from_trits!(u128);
unsigned_try_from_trits!(u64);
unsigned_try_from_trits!(u32);
unsigned_try_from_trits!(u16);
unsigned_try_from_trits!(u8);

/// Attempt to convert the given trit slice into a number. If the numeric representation of the
/// trit slice is too large or small to fit the numeric type, or does not contain any trits, an
/// error will be returned.
pub fn trits_to_int<I, T: RawEncoding + ?Sized>(trits: &Trits<T>) -> Result<I, Error>
where
    I: Clone + CheckedAdd + CheckedSub + PartialOrd + Num,
{
    if trits.is_empty() {
        Err(Error::Empty)
    } else {
        let mut acc = I::zero();

        for trit in trits.iter().rev() {
            let old_acc = acc.clone();
            acc = trit
                .add_to_num(acc)?
                .checked_add(&old_acc)
                .and_then(|acc| acc.checked_add(&old_acc))
                .ok_or_else(|| {
                    if old_acc < I::zero() {
                        Error::Underflow
                    } else {
                        Error::Overflow
                    }
                })?;
        }

        Ok(acc)
    }
}

/// Produce an iterator over the [`Btrit`]s that make up a given integer.
pub fn signed_int_trits<I>(x: I) -> impl Iterator<Item = Btrit> + Clone
where
    I: Clone + AsPrimitive<i8> + FromPrimitive + Signed,
{
    let is_neg = x.is_negative();
    let mut x = if is_neg { x } else { -x };

    let radix = I::from_i8(3).unwrap();

    std::iter::from_fn(move || {
        if x.is_zero() {
            None
        } else {
            let modulus = ((x + I::one()).abs() % radix).as_();
            x = x / radix;
            if modulus == 1 {
                x = x + -I::one();
            }
            Some(Btrit::try_from(((modulus + 2) % 3 - 1) * if is_neg { -1 } else { 1 }).unwrap())
        }
    })
    // If the integer is exactly 0, add an extra trit
    .chain(Some(Btrit::Zero).filter(|_| x.is_zero()))
}

/// Produce an iterator over the [`Utrit`]s that make up a given integer.
pub fn unsigned_int_trits<I>(mut x: I) -> impl Iterator<Item = Utrit> + Clone
where
    I: Clone + AsPrimitive<u8> + FromPrimitive + Num,
{
    let radix = I::from_u8(3).unwrap();

    std::iter::from_fn(move || {
        if x.is_zero() {
            None
        } else {
            let modulus = (x % radix).as_();
            x = x / radix;
            Some(Utrit::try_from(modulus).unwrap())
        }
    })
    // If the integer is exactly 0, add an extra trit
    .chain(Some(Utrit::Zero).filter(|_| x.is_zero()))
}
