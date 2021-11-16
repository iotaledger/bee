// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::*;
use bee_test::ternary::*;

use rand::prelude::*;

fn create_generic<T: raw::RawEncodingBuf>() {
    assert!(TritBuf::<T>::new().len() == 0);
    fuzz(100, || {
        let len = thread_rng().gen_range(0..100);
        let mut buf = TritBuf::<T>::zeros(len);
        assert!(buf.len() == len);
        buf.clear();
        assert!(buf.len() == 0);
    });
    fuzz(100, || {
        let trits = gen_buf_balanced::<T>(0..1000).1;
        let buf: TritBuf<T> = trits
            .iter()
            .map(|t| <T::Slice as raw::RawEncoding>::Trit::try_from(*t).ok().unwrap())
            .collect();
        assert!(buf.len() == trits.len());
    });
}

fn create_unbalanced<T: raw::RawEncodingBuf>() {
    assert!(TritBuf::<T>::new().len() == 0);
    fuzz(100, || {
        let len = thread_rng().gen_range(0..100);
        assert!(TritBuf::<T>::zeros(len).len() == len);
    });
    fuzz(100, || {
        let trits = gen_buf_unbalanced::<T>(0..1000).1;
        let buf: TritBuf<T> = trits
            .iter()
            .map(|t| <T::Slice as raw::RawEncoding>::Trit::try_from(*t).ok().unwrap())
            .collect();
        assert!(buf.len() == trits.len());
    });
}

fn push_pop_generic<T: raw::RawEncodingBuf>() {
    fuzz(100, || {
        let (mut a, mut b) = gen_buf_balanced::<T>(0..100);

        for _ in 0..1000 {
            if thread_rng().gen() {
                let trit = gen_trit_balanced();
                a.push(trit.try_into().unwrap_or_else(|_| unreachable!()));
                b.push(trit);
            } else {
                assert_eq!(
                    a.pop(),
                    b.pop().map(|x| x.try_into().unwrap_or_else(|_| unreachable!()))
                );
            }
        }
    });
}

fn push_pop_generic_unbalanced<T: raw::RawEncodingBuf>() {
    fuzz(100, || {
        let (mut a, mut b) = gen_buf_unbalanced::<T>(0..100);

        for _ in 0..1000 {
            if thread_rng().gen() {
                let trit = gen_trit_unbalanced();
                a.push(trit.try_into().unwrap_or_else(|_| unreachable!()));
                b.push(trit);
            } else {
                assert_eq!(
                    a.pop(),
                    b.pop().map(|x| x.try_into().unwrap_or_else(|_| unreachable!()))
                );
            }
        }
    });
}

fn eq_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let a = gen_buf_balanced::<T>(0..1000).0;
        let b = a.clone();

        assert_eq!(a, b);
    });
}

fn eq_generic_unbalanced<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let a = gen_buf_unbalanced::<T>(0..1000).0;
        let b = a.clone();

        assert_eq!(a, b);
    });
}

fn encode_generic<T: raw::RawEncodingBuf + Clone, U: raw::RawEncodingBuf>()
where
    U::Slice: raw::RawEncoding<Trit = <T::Slice as raw::RawEncoding>::Trit>,
{
    fuzz(100, || {
        let a = gen_buf_balanced::<T>(0..100).0;
        let b = a.encode::<U>();

        assert_eq!(a, b);
        assert_eq!(a.len(), b.len());

        let c = b.encode::<T>();

        assert_eq!(a, c);
        assert_eq!(a.len(), c.len());
    });
}

fn with_capacity_generic<T: raw::RawEncodingBuf>() {
    let cap = 243; // TODO: Use random capacity
    let mut buf = TritBuf::<T>::with_capacity(cap);
    assert_eq!(buf.capacity(), cap);
    for _ in 0..cap {
        buf.push(<T::Slice as raw::RawEncoding>::Trit::zero());
    }
    assert_eq!(buf.capacity(), cap);
}

#[test]
fn create() {
    create_generic::<T1B1Buf<Btrit>>();
    create_unbalanced::<T1B1Buf<Utrit>>();
    create_generic::<T2B1Buf>();
    create_generic::<T3B1Buf>();
    create_generic::<T4B1Buf>();
    create_generic::<T5B1Buf>();
}

#[test]
fn push_pop() {
    push_pop_generic::<T1B1Buf<Btrit>>();
    push_pop_generic_unbalanced::<T1B1Buf<Utrit>>();
    push_pop_generic::<T2B1Buf>();
    push_pop_generic::<T3B1Buf>();
    push_pop_generic::<T4B1Buf>();
    push_pop_generic::<T5B1Buf>();
}

#[test]
fn eq() {
    eq_generic::<T1B1Buf<Btrit>>();
    eq_generic_unbalanced::<T1B1Buf<Utrit>>();
    eq_generic::<T2B1Buf>();
    eq_generic::<T3B1Buf>();
    eq_generic::<T4B1Buf>();
    eq_generic::<T5B1Buf>();
}

#[test]
fn encode() {
    encode_generic::<T1B1Buf<Btrit>, T2B1Buf>();
    // encode_generic::<T1B1Buf<Utrit>, T2B1Buf>();
    encode_generic::<T1B1Buf<Btrit>, T3B1Buf>();
    // encode_generic::<T1B1Buf<Utrit>, T3B1Buf>();
    encode_generic::<T1B1Buf<Btrit>, T4B1Buf>();
    // encode_generic::<T1B1Buf<Utrit>, T4B1Buf>();
    encode_generic::<T2B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T2B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T3B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T3B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T4B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T4B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T5B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T5B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T2B1Buf, T3B1Buf>();
    encode_generic::<T3B1Buf, T4B1Buf>();
    encode_generic::<T3B1Buf, T5B1Buf>();
    encode_generic::<T3B1Buf, T2B1Buf>();
    encode_generic::<T2B1Buf, T3B1Buf>();
    encode_generic::<T4B1Buf, T2B1Buf>();
    encode_generic::<T4B1Buf, T3B1Buf>();
    encode_generic::<T5B1Buf, T2B1Buf>();
    encode_generic::<T5B1Buf, T3B1Buf>();
}

#[test]
fn with_capacity() {
    with_capacity_generic::<T1B1Buf<Btrit>>();
    with_capacity_generic::<T1B1Buf<Utrit>>();
    with_capacity_generic::<T2B1Buf>();
    with_capacity_generic::<T3B1Buf>();
    with_capacity_generic::<T4B1Buf>();
    with_capacity_generic::<T5B1Buf>();
}
