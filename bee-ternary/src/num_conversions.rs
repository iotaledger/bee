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

use std::convert::TryFrom;

pub use crate::{
    t1b1::{T1B1Buf, T1B1},
    trit::{Btrit, ShiftTernary, Trit, Utrit},
    TritBuf,
};

use std::convert::From;

const RADIX: u8 = 3;
const MAX_TRITS_IN_I64: usize = (0.63 * 64_f32) as usize + 1; // (log2/log3)*64

fn min_trits(value: u64) -> usize {
    let mut num = 1;
    let mut vp: u64 = 1;

    while value > vp {
        vp = vp * RADIX as u64 + 1;
        num += 1;
    }

    num
}

// TODO - generalize over all encodings
// Impl copied from:
// [https://github.com/iotaledger/iota_common/blob/master/common/trinary/trit_long.c#L62]
impl From<i64> for TritBuf<T1B1Buf> {
    fn from(value: i64) -> Self {
        let negative = value < 0;
        // Edge case where value == i64::MIN. In this case,
        // "abs" cannot return a value greater than i64::MAX
        // Which it should since the range is not symmetrical
        // so we "force" the (u64) value explicitly
        let mut value_abs = match value {
            std::i64::MIN => (std::i64::MAX as u64 + 1),
            _ => value.abs() as u64,
        };

        let size = min_trits(value_abs);
        let mut buf = Self::zeros(size);

        for pos in 0..size {
            if value_abs == 0 {
                break;
            }
            let mut curr_trit = ((value_abs + 1) % (RADIX as u64)) as i8 - 1;
            if negative {
                curr_trit = -curr_trit;
            }

            // This can not fail because curr_trit is "some_value % 3 - 1" which is always within range
            buf.set(pos, Btrit::try_from(curr_trit).unwrap());

            value_abs += 1;
            value_abs /= RADIX as u64;
        }

        buf
    }
}

#[derive(Debug, PartialEq)]
pub enum TritsI64ConversionError {
    EmptyTrits,
    AbsValueTooBig,
}

// TODO - generalize over all encodings
// Impl copied from:
// [https://github.com/iotaledger/iota_common/blob/1b56a5282933fb674181001630e7b2e2c33b5eea/common/trinary/trit_long.c#L31]
impl TryFrom<TritBuf<T1B1Buf>> for i64 {
    type Error = TritsI64ConversionError;

    fn try_from(trits: TritBuf<T1B1Buf>) -> Result<Self, Self::Error> {
        if trits.len() == 0 {
            return Err(TritsI64ConversionError::EmptyTrits);
        }

        if trits.len() > MAX_TRITS_IN_I64 {
            // TODO test
            for index in MAX_TRITS_IN_I64..trits.len() {
                if trits.get(index).unwrap() != Btrit::Zero {
                    return Err(TritsI64ConversionError::AbsValueTooBig);
                }
            }
        }

        let mut accum: i64 = 0;
        // TODO change end if ending 0s
        let end = trits.len();

        if trits.len() >= MAX_TRITS_IN_I64 {
            let mut accum_i128: i128 = 0;
            for index in (0..end).rev() {
                let trit_value = match trits.get(index).unwrap() {
                    Btrit::Zero => 0,
                    Btrit::NegOne => -1,
                    Btrit::PlusOne => 1,
                };
                accum_i128 = (accum_i128 * RADIX as i128) as i128 + trit_value;
                if accum_i128 > 0 && accum_i128 > i64::max_value() as i128
                    || accum_i128 < 0 && accum_i128 < i64::min_value() as i128 - 1
                {
                    return Err(TritsI64ConversionError::AbsValueTooBig);
                }
                accum = accum_i128 as i64;
            }
        } else {
            for index in (0..end).rev() {
                let trit_value = match trits.get(index).unwrap() {
                    Btrit::Zero => 0,
                    Btrit::NegOne => -1,
                    Btrit::PlusOne => 1,
                };
                accum = (accum * RADIX as i64) as i64 + trit_value;
            }
        }

        Ok(accum)
    }
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
        let buff = TritBuf::<T1B1Buf>::zeros(0);
        match i64::try_from(buff) {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e, TritsI64ConversionError::EmptyTrits),
        }
    }

    #[test]
    fn convert_1_to_trits() {
        let num = 1;
        let buff = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }

    #[test]
    fn convert_neg_1_to_trits() {
        let num = -1;
        let buff = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }

    #[test]
    fn convert_i64_max_to_trits() {
        let num = std::i64::MAX;
        let buff = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }

    #[test]
    fn convert_i64_min_to_trits() {
        let num = std::i64::MIN;
        let buff = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }

    #[test]
    fn convert_range_to_trits() {
        let now = Instant::now();
        for num in -100_000..100_000 {
            let buff = TritBuf::<T1B1Buf>::try_from(num);
            let converted_num = i64::try_from(buff.unwrap()).unwrap();
            assert_eq!(converted_num, num);
        }
        let message = format!("\nconvert_range_to_trits Elapsed: {}\n", now.elapsed().as_secs_f64());
        io::stdout().write_all(message.as_bytes()).unwrap();
    }

    #[test]
    fn error_on_num_too_big() {
        let buff = TritBuf::<T1B1Buf>::filled(MAX_TRITS_IN_I64, Btrit::PlusOne);
        assert_eq!(i64::try_from(buff).is_ok(), false);
    }
}
