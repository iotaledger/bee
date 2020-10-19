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

use bee_ternary::{
    convert::*,
    trit::{Btrit, Utrit},
    T1B1Buf, TritBuf,
};
use std::{
    convert::TryFrom,
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
