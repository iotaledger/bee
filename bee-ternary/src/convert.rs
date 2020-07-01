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

use crate::{Btrit, RawEncoding, RawEncodingBuf, TritBuf, Trits};
use num_traits::{AsPrimitive, Bounded, FromPrimitive, One, ToPrimitive, Zero};
use std::{
    convert::TryFrom,
    ops::{Add, Mul, Neg},
};

#[derive(Debug, PartialEq)]
pub enum Error {
    Empty,
    TooBig,
}

impl<I: AsPrimitive<i64>, T: RawEncodingBuf> From<I> for TritBuf<T>
where
    T::Slice: RawEncoding<Trit = Btrit>,
{
    fn from(x: I) -> Self {
        integer_trits(x).collect()
    }
}

macro_rules! try_from_trits {
    ($int:ident) => {
        impl<'a, T: RawEncoding<Trit = Btrit> + ?Sized> TryFrom<&'a Trits<T>> for $int {
            type Error = Error;
            fn try_from(trits: &'a Trits<T>) -> Result<Self, Self::Error> {
                trits_to_int(trits)
            }
        }
    };
}

try_from_trits!(i64);
try_from_trits!(i32);
try_from_trits!(i16);
try_from_trits!(i8);

pub(crate) fn max_trits<I: ToPrimitive + Bounded>() -> usize {
    (0.63 * std::mem::size_of::<I>() as f32 * 8.0) as usize + 1 // (log2/log3)*64
}

fn trits_to_int<I, T: RawEncoding<Trit = Btrit> + ?Sized>(trits: &Trits<T>) -> Result<I, Error>
where
    I: Copy + FromPrimitive + ToPrimitive + Bounded + Zero + One + Neg<Output = I> + Add + Mul,
{
    if trits.is_empty() {
        Err(Error::Empty)
    } else {
        let max_trits: usize = max_trits::<I>();

        if trits.len() > max_trits {
            for index in max_trits..trits.len() {
                if trits.get(index).unwrap() != Btrit::Zero {
                    return Err(Error::TooBig);
                }
            }
        }

        let mut acc = I::zero();
        // TODO change end if ending 0s
        let end = trits.len().min(max_trits);

        if trits.len() >= max_trits {
            let mut acc_i128: i128 = 0;
            for index in (0..end).rev() {
                let trit_value = match trits.get(index).unwrap() {
                    Btrit::Zero => 0,
                    Btrit::NegOne => -1,
                    Btrit::PlusOne => 1,
                };
                acc_i128 = (acc_i128 * RADIX as i128) as i128 + trit_value;
                if acc_i128 > 0 && acc_i128 > I::max_value().to_i128().unwrap()
                    || acc_i128 < 0 && acc_i128 < I::min_value().to_i128().unwrap() - 1
                {
                    return Err(Error::TooBig);
                }
                acc = I::from_i128(acc_i128).unwrap();
            }
        } else {
            for index in (0..end).rev() {
                let trit_value = match trits.get(index).unwrap() {
                    Btrit::Zero => I::zero(),
                    Btrit::NegOne => -I::one(),
                    Btrit::PlusOne => I::one(),
                };
                acc = (acc * I::from_u8(RADIX).unwrap()) + trit_value;
            }
        }

        Ok(acc)
    }
}

const RADIX: u8 = 3;

pub fn integer_trits<I: AsPrimitive<i64>>(x: I) -> impl Iterator<Item = Btrit> {
    let x = x.as_();
    let is_neg = x < 0;

    let mut x_abs = if x == i64::MIN {
        i64::MAX as u64 + 1
    } else {
        x.abs() as u64
    };

    std::iter::from_fn(move || {
        if x_abs == 0 {
            None
        } else {
            let trit = match ((x_abs + 1) % RADIX as u64) as i8 - 1 {
                x if is_neg => -x,
                x => x,
            };

            x_abs += 1;
            x_abs /= RADIX as u64;

            Some(Btrit::try_from(trit).unwrap())
        }
    })
    // If the integer is exactly 0, add an extra trit
    .chain(Some(Btrit::Zero).filter(|_| x == 0))
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
        match i64::try_from(buf.as_slice()) {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e, Error::Empty),
        }
    }

    #[test]
    fn round_robin() {
        let nums = [1, -1, i64::MAX, i64::MIN];
        for n in &nums {
            assert_eq!(i64::try_from(TritBuf::<T1B1Buf>::from(*n).as_slice()).unwrap(), *n);
        }
    }

    #[test]
    fn convert_range_to_trits() {
        let now = Instant::now();
        for num in -100_000..100_000i64 {
            let buf = TritBuf::<T1B1Buf>::from(num);
            let converted_num = i64::try_from(buf.as_slice()).unwrap();
            assert_eq!(converted_num, num);
        }
        let message = format!("\nconvert_range_to_trits Elapsed: {}\n", now.elapsed().as_secs_f64());
        io::stdout().write_all(message.as_bytes()).unwrap();
    }

    #[test]
    fn error_on_num_too_big() {
        let buf = TritBuf::<T1B1Buf>::filled(max_trits::<i64>(), Btrit::PlusOne);
        assert_eq!(i64::try_from(buf.as_slice()).is_ok(), false);
    }

    #[test]
    fn max_trits_types() {
        assert!(max_trits::<u64>() == 41, "{}", max_trits::<u64>());
        assert!(max_trits::<u32>() == 21, "{}", max_trits::<u32>());
        assert!(max_trits::<u16>() == 11, "{}", max_trits::<u16>());
        assert!(max_trits::<u8>() == 6, "{}", max_trits::<u8>());
    }

    #[test]
    fn max_trits_zero() {
        let make_trits = |trits| TritBuf::<T1B1Buf>::from_i8s(trits).unwrap();
        assert!(trits_to_int::<i8, _>(&make_trits(&[1, 0, -1, -1, -1, 1, 0, 0, 0])).is_ok());
        assert!(trits_to_int::<i8, _>(&make_trits(&[1, 0, -1, -1, -1, 1, 1, 0, 0])).is_err());
    }
}
