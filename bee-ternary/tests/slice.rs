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

use std::convert::{TryFrom, TryInto};

mod common;
use self::common::*;

use bee_ternary::*;
use rand::prelude::*;

fn get_generic<T: raw::RawEncodingBuf + Clone>() {
    println!("{}", std::any::type_name::<T>());
    fuzz(100, || {
        let (a, a_i8) = gen_buf::<T>(1..1000);

        fuzz(25, || {
            assert_eq!(a.get(a.len() + thread_rng().gen_range(0, 20)), None);
        });

        let mut sl = a.as_slice();
        let mut sl_i8 = &a_i8[..];
        for _ in 0..20 {
            if sl.len() == 0 {
                break;
            }
            let i = thread_rng().gen_range(0, sl.len());
            assert_eq!(
                sl.get(i),
                Some(<T::Slice as raw::RawEncoding>::Trit::try_from(sl_i8[i]).unwrap_or_else(|_| unreachable!())),
            );

            let idx = thread_rng().gen_range(0, sl.len());
            let len = thread_rng().gen_range(0, sl.len() - idx);
            sl_i8 = &sl_i8[idx..idx + len];
            sl = &sl[idx..idx + len];
        }
    });
}

fn get_generic_unbalanced<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (a, a_i8) = gen_buf_unbalanced::<T>(1..1000);

        fuzz(25, || {
            assert_eq!(a.get(a.len() + thread_rng().gen_range(0, 20)), None);
        });

        let mut sl = a.as_slice();
        let mut sl_i8 = &a_i8[..];
        for _ in 0..20 {
            if sl.len() == 0 {
                break;
            }
            let i = thread_rng().gen_range(0, sl.len());
            assert_eq!(
                sl.get(i),
                Some(<T::Slice as raw::RawEncoding>::Trit::try_from(sl_i8[i]).unwrap_or_else(|_| unreachable!())),
            );

            let idx = thread_rng().gen_range(0, sl.len());
            let len = thread_rng().gen_range(0, sl.len() - idx);
            sl_i8 = &sl_i8[idx..idx + len];
            sl = &sl[idx..idx + len];
        }
    });
}

fn set_generic<T: raw::RawEncodingBuf + Clone>() {
    println!("{}", std::any::type_name::<T>());
    fuzz(100, || {
        let (mut a, mut a_i8) = gen_buf::<T>(1..1000);

        fuzz(100, || {
            let mut sl = a.as_slice_mut();
            let mut sl_i8 = &mut a_i8[..];
            for _ in 0..10 {
                if sl.len() == 0 {
                    break;
                }

                let i = thread_rng().gen_range(0, sl.len());
                let trit = thread_rng().gen_range(-1i8, 2);

                sl.set(i, trit.try_into().unwrap_or_else(|_| unreachable!()));
                sl_i8[i] = trit;

                assert_eq!(
                    sl.get(i),
                    Some(<T::Slice as raw::RawEncoding>::Trit::try_from(sl_i8[i]).unwrap_or_else(|_| unreachable!())),
                );

                let idx = thread_rng().gen_range(0, sl.len());
                let len = thread_rng().gen_range(0, sl.len() - idx);
                sl_i8 = &mut sl_i8[idx..idx + len];
                sl = &mut sl[idx..idx + len];
            }

            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == (*b).try_into().unwrap_or_else(|_| unreachable!())));

            assert_eq!(a.len(), a_i8.len());
        });
    });
}

fn set_generic_unbalanced<T: raw::RawEncodingBuf + Clone>() {
    println!("{}", std::any::type_name::<T>());
    fuzz(100, || {
        let (mut a, mut a_i8) = gen_buf_unbalanced::<T>(1..1000);

        fuzz(100, || {
            let mut sl = a.as_slice_mut();
            let mut sl_i8 = &mut a_i8[..];
            for _ in 0..10 {
                if sl.len() == 0 {
                    break;
                }

                let i = thread_rng().gen_range(0, sl.len());
                let trit = thread_rng().gen_range(0i8, 3);

                sl.set(i, trit.try_into().unwrap_or_else(|_| unreachable!()));
                sl_i8[i] = trit;

                assert_eq!(
                    sl.get(i),
                    Some(<T::Slice as raw::RawEncoding>::Trit::try_from(sl_i8[i]).unwrap_or_else(|_| unreachable!())),
                );

                let idx = thread_rng().gen_range(0, sl.len());
                let len = thread_rng().gen_range(0, sl.len() - idx);
                sl_i8 = &mut sl_i8[idx..idx + len];
                sl = &mut sl[idx..idx + len];
            }

            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == (*b).try_into().unwrap_or_else(|_| unreachable!())));

            assert_eq!(a.len(), a_i8.len());
        });
    });
}

fn chunks_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (a, a_i8) = gen_buf::<T>(2..1000);

        let chunk_len = thread_rng().gen_range(1, a.len());
        for (a, a_i8) in a.chunks(chunk_len).zip(a_i8.chunks(chunk_len)) {
            assert_eq!(a.len(), a_i8.len());
            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == (*b).try_into().unwrap_or_else(|_| unreachable!())));
        }
    });
}

fn chunks_generic_unbalanced<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (a, a_i8) = gen_buf_unbalanced::<T>(2..1000);

        let chunk_len = thread_rng().gen_range(1, a.len());
        for (a, a_i8) in a.chunks(chunk_len).zip(a_i8.chunks(chunk_len)) {
            assert_eq!(a.len(), a_i8.len());
            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == (*b).try_into().unwrap_or_else(|_| unreachable!())));
        }
    });
}

fn set_panic_generic<T: raw::RawEncodingBuf + Clone>() {
    let mut a = gen_buf::<T>(0..1000).0;
    let len = a.len();
    a.set(len, <T::Slice as raw::RawEncoding>::Trit::zero());
}

fn set_panic_generic_unbalanced<T: raw::RawEncodingBuf + Clone>() {
    let mut a = gen_buf_unbalanced::<T>(0..1000).0;
    let len = a.len();
    a.set(len, <T::Slice as raw::RawEncoding>::Trit::zero());
}

#[test]
fn get() {
    get_generic::<T1B1Buf<Btrit>>();
    get_generic_unbalanced::<T1B1Buf<Utrit>>();
    get_generic::<T2B1Buf>();
    get_generic::<T3B1Buf>();
    get_generic::<T4B1Buf>();
    get_generic::<T5B1Buf>();
}

#[test]
fn set() {
    set_generic::<T1B1Buf<Btrit>>();
    set_generic_unbalanced::<T1B1Buf<Utrit>>();
    set_generic::<T2B1Buf>();
    set_generic::<T3B1Buf>();
    set_generic::<T4B1Buf>();
    set_generic::<T5B1Buf>();
}

#[test]
#[should_panic]
fn set_panic() {
    set_panic_generic::<T1B1Buf<Btrit>>();
    set_panic_generic_unbalanced::<T1B1Buf<Utrit>>();
    set_panic_generic::<T2B1Buf>();
    set_panic_generic::<T3B1Buf>();
    set_panic_generic::<T4B1Buf>();
    set_panic_generic::<T5B1Buf>();
}

#[test]
fn chunks() {
    chunks_generic::<T1B1Buf<Btrit>>();
    chunks_generic_unbalanced::<T1B1Buf<Utrit>>();
    chunks_generic::<T2B1Buf>();
    chunks_generic::<T3B1Buf>();
    chunks_generic::<T4B1Buf>();
    chunks_generic::<T5B1Buf>();
}

#[test]
fn chunks_mut() {
    fuzz(100, || {
        let (mut a, mut a_i8) = gen_buf::<T1B1Buf>(2..1000);

        let chunk_len = thread_rng().gen_range(1, a.len());
        for (a, a_i8) in a.chunks_mut(chunk_len).zip(a_i8.chunks_mut(chunk_len)) {
            assert_eq!(a.len(), a_i8.len());
            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == (*b).try_into().unwrap_or_else(|_| unreachable!())));
        }
    });
}
