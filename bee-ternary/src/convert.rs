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

use crate::{Btrit, RawEncoding, RawEncodingBuf, TritBuf, Trits, Utrit};
use num_traits::{AsPrimitive, CheckedAdd, FromPrimitive, Signed, Num};
use std::convert::TryFrom;

/// An error that may be produced during numeric conversion
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The trit slice didn't contain enough trits to be considered a numeric value.
    Empty,
    /// The trit slice has a numeric value that's too big to contain within the requested numeric
    /// type.
    TooBig,
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
                trits_to_signed_int(trits)
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
signed_try_from_trits!(i64);
signed_try_from_trits!(i32);
signed_try_from_trits!(i16);
signed_try_from_trits!(i8);

macro_rules! unsigned_try_from_trits {
    ($int:ident) => {
        impl<'a, T: RawEncoding<Trit = Utrit> + ?Sized> TryFrom<&'a Trits<T>> for $int {
            type Error = Error;
            fn try_from(trits: &'a Trits<T>) -> Result<Self, Self::Error> {
                trits_to_unsigned_int(trits)
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
unsigned_try_from_trits!(u64);
unsigned_try_from_trits!(u32);
unsigned_try_from_trits!(u16);
unsigned_try_from_trits!(u8);

/// Attempt to convert the given trit slice into a number. If the numeric representation of the
/// trit slice is too large or small to fit the numeric type, or does not contain any trits, an
/// error will be returned.
pub fn trits_to_signed_int<I, T: RawEncoding<Trit = Btrit> + ?Sized>(trits: &Trits<T>) -> Result<I, Error>
where
    I: Clone + CheckedAdd + Signed,
{
    if trits.is_empty() {
        Err(Error::Empty)
    } else {
        let mut acc = I::zero();

        for trit in trits.iter().rev() {
            let old_acc = acc.clone();
            acc = match trit {
                Btrit::NegOne => -I::one(),
                Btrit::Zero => I::zero(),
                Btrit::PlusOne => I::one(),
            }
            .checked_add(&old_acc)
            .and_then(|acc| acc.checked_add(&old_acc))
            .and_then(|acc| acc.checked_add(&old_acc))
            .ok_or(Error::TooBig)?;
        }

        Ok(acc)
    }
}

/// Attempt to convert the given trit slice into a number. If the numeric representation of the
/// trit slice is too large to fit the desired integer, or does not contain any trits, an error
/// will be returned.
pub fn trits_to_unsigned_int<I, T: RawEncoding<Trit = Utrit> + ?Sized>(trits: &Trits<T>) -> Result<I, Error>
where
    I: Clone + CheckedAdd + Num,
{
    if trits.is_empty() {
        Err(Error::Empty)
    } else {
        let mut acc = I::zero();

        for trit in trits.iter().rev() {
            let old_acc = acc.clone();
            acc = match trit {
                Utrit::Zero => I::zero(),
                Utrit::One => I::one(),
                Utrit::Two => I::one() + I::one(),
            }
            .checked_add(&old_acc)
            .and_then(|acc| acc.checked_add(&old_acc))
            .and_then(|acc| acc.checked_add(&old_acc))
            .ok_or(Error::TooBig)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::T1B1Buf;

    use std::{
        io::{self, Write},
        time::Instant,
    };

    #[test]
    fn error_empty_trits() {
        let buf = TritBuf::<T1B1Buf>::zeros(0);
        assert_eq!(i64::try_from(buf.as_slice()).unwrap_err(), Error::Empty);

        let buf = TritBuf::<T1B1Buf<Utrit>>::zeros(0);
        assert_eq!(u64::try_from(buf.as_slice()).unwrap_err(), Error::Empty);
    }

    #[test]
    fn signed_round_robin() {
        let nums = [
            0,
            1,
            -1,
            42,
            -42,
            7331,
            -7331,
            i64::MAX - 1,
            i64::MIN + 1,
            i64::MAX,
            i64::MIN,
        ];
        for n in &nums {
            let x = TritBuf::<T1B1Buf>::from(*n);
            let new = i64::try_from(x.as_slice()).unwrap();
            assert_eq!(new, *n);
        }
    }

    #[test]
    fn unsigned_round_robin() {
        let nums = [0, 1, 42, 7331, u64::MAX - 1, u64::MAX];
        for n in &nums {
            let x = TritBuf::<T1B1Buf<_>>::from(*n);
            let new = u64::try_from(x.as_slice()).unwrap();
            assert_eq!(new, *n);
        }
    }

    #[test]
    fn signed_range_to_trits() {
        let now = Instant::now();
        for num in -100_000..100_001i64 {
            let buf = TritBuf::<T1B1Buf>::from(num);
            let converted_num = i64::try_from(buf.as_slice()).unwrap();
            assert_eq!(converted_num, num, "num {}, trits {}", num, buf.as_slice());
        }
        let message = format!("\nconvert_range_to_trits Elapsed: {}\n", now.elapsed().as_secs_f64());
        io::stdout().write_all(message.as_bytes()).unwrap();
    }

    #[test]
    fn unsigned_range_to_trits() {
        let now = Instant::now();
        for num in 0..100_001u64 {
            let buf = TritBuf::<T1B1Buf<_>>::from(num);
            let converted_num = u64::try_from(buf.as_slice()).unwrap();
            assert_eq!(converted_num, num, "num {}, trits {}", num, buf.as_slice());
        }
        let message = format!("\nconvert_range_to_trits Elapsed: {}\n", now.elapsed().as_secs_f64());
        io::stdout().write_all(message.as_bytes()).unwrap();
    }

    #[test]
    fn error_on_num_too_big() {
        let buf = TritBuf::<T1B1Buf>::filled(41, Btrit::PlusOne);
        assert_eq!(i64::try_from(buf.as_slice()).is_ok(), false);

        let buf = TritBuf::<T1B1Buf<_>>::filled(42, Utrit::One);
        assert_eq!(u64::try_from(buf.as_slice()).is_ok(), false);
    }

    #[test]
    fn signed_int_to_trits() {
        let make_trits = |trits| TritBuf::<T1B1Buf>::from_i8s(trits).unwrap();

        let tests = [
            (0, make_trits(&[0])),
            (45, make_trits(&[0, 0, -1, -1, 1])),
            (
                3777554354,
                make_trits(&[-1, -1, -1, -1, 1, 1, -1, 0, -1, 1, 1, 0, 1, -1, 1, -1, 1, -1, 1, 0, 1]),
            ),
            (
                522626226,
                make_trits(&[0, -1, 0, -1, 1, 1, 1, 1, 0, -1, 1, 1, -1, 1, 1, 0, 0, 1, 1]),
            ),
        ];

        for (n, buf) in &tests {
            let neg_buf = -buf.clone(); // Test the negation too

            assert_eq!(&TritBuf::<T1B1Buf>::from(*n), buf);
            assert_eq!(&TritBuf::<T1B1Buf>::from(-*n), &neg_buf);

            assert_eq!(i64::try_from(buf.as_slice()).unwrap(), *n);
            assert_eq!(i64::try_from(neg_buf.as_slice()).unwrap(), -*n);
        }
    }

    #[test]
    fn unsigned_int_to_trits() {
        let make_trits = |trits| TritBuf::<T1B1Buf<Utrit>>::from_u8s(trits).unwrap();

        let tests = [
            (0, make_trits(&[0])),
            (45, make_trits(&[0, 0, 2, 1])),
            (
                3777554354,
                make_trits(&[2, 1, 1, 1, 0, 1, 2, 2, 1, 0, 1, 0, 1, 2, 0, 2, 0, 2, 0, 0, 1]),
            ),
            (
                522626226,
                make_trits(&[0, 2, 2, 1, 0, 1, 1, 1, 0, 2, 0, 1, 2, 0, 1, 0, 0, 1, 1]),
            ),
        ];

        for (n, buf) in &tests {
            assert_eq!(&TritBuf::<T1B1Buf<_>>::from(*n), buf);
            assert_eq!(u64::try_from(buf.as_slice()).unwrap(), *n);
        }
    }

    #[test]
    fn signed_max_trits_zero() {
        let make_trits = |trits| TritBuf::<T1B1Buf>::from_i8s(trits).unwrap();

        // Maximum i8
        assert!(i8::try_from(make_trits(&[1, 0, -1, -1, -1, 1]).as_slice()).is_ok());
        // Overflow!
        assert!(i8::try_from(make_trits(&[-1, 1, -1, -1, -1, 1]).as_slice()).is_err());
        // Minimum i8 (we can go back to -128 because 2's complement repr)
        assert!(i8::try_from(make_trits(&[1, -1, 1, 1, 1, -1]).as_slice()).is_ok());
        // Underflow!
        assert!(i8::try_from(make_trits(&[0, -1, 1, 1, 1, -1]).as_slice()).is_err());
        // Zero-padded
        assert!(i8::try_from(make_trits(&[1, 0, -1, -1, -1, 1, 0, 0, 0]).as_slice()).is_ok());
        // All the zeros
        assert_eq!(
            i8::try_from(make_trits(&[0, 0, 0, 0, 0, 0, 0, 0, 0]).as_slice()).ok(),
            Some(0)
        );
    }

    #[test]
    fn unsigned_max_trits_zero() {
        let make_trits = |trits| TritBuf::<T1B1Buf<Utrit>>::from_u8s(trits).unwrap();

        // Maximum u8
        assert!(u8::try_from(make_trits(&[0, 1, 1, 0, 0, 1]).as_slice()).is_ok());
        // Overflow!
        assert!(u8::try_from(make_trits(&[1, 1, 1, 0, 0, 1]).as_slice()).is_err());
        // Zero-padded
        assert!(u8::try_from(make_trits(&[0, 1, 1, 0, 0, 1, 0, 0, 0]).as_slice()).is_ok());
        // All the zeros
        assert_eq!(
            u8::try_from(make_trits(&[0, 0, 0, 0, 0, 0, 0, 0, 0]).as_slice()).ok(),
            Some(0)
        );
    }
}
