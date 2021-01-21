// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::{raw::*, *};
use bee_test::ternary::*;

use serde::{de::DeserializeOwned, *};

fn serialize_generic<T: raw::RawEncodingBuf>()
where
    <T::Slice as RawEncoding>::Trit: Serialize,
{
    let (a, a_i8) = gen_buf::<T>(0..1000);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        format!("[{}]", a_i8.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")),
    );
}

fn serialize_generic_unbalanced<T: raw::RawEncodingBuf>()
where
    <T::Slice as RawEncoding>::Trit: Serialize,
{
    let (a, a_i8) = gen_buf_unbalanced::<T>(0..1000);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        format!("[{}]", a_i8.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")),
    );
}

fn deserialize_generic<T: raw::RawEncodingBuf>()
where
    <T::Slice as RawEncoding>::Trit: DeserializeOwned,
{
    let (a, a_i8) = gen_buf::<T>(0..1000);
    assert_eq!(
        serde_json::from_str::<TritBuf<T>>(&format!(
            "[{}]",
            a_i8.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")
        ))
        .unwrap(),
        a,
    );
}

fn deserialize_generic_unbalanced<T: raw::RawEncodingBuf>()
where
    <T::Slice as RawEncoding>::Trit: DeserializeOwned,
{
    let (a, a_i8) = gen_buf_unbalanced::<T>(0..1000);
    assert_eq!(
        serde_json::from_str::<TritBuf<T>>(&format!(
            "[{}]",
            a_i8.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")
        ))
        .unwrap(),
        a,
    );
}

#[test]
fn serialize() {
    serialize_generic::<T1B1Buf<Btrit>>();
    serialize_generic_unbalanced::<T1B1Buf<Utrit>>();
    serialize_generic::<T2B1Buf>();
    serialize_generic::<T3B1Buf>();
    serialize_generic::<T4B1Buf>();
    serialize_generic::<T5B1Buf>();
}

#[test]
fn deserialize() {
    deserialize_generic::<T1B1Buf<Btrit>>();
    deserialize_generic_unbalanced::<T1B1Buf<Utrit>>();
    deserialize_generic::<T2B1Buf>();
    deserialize_generic::<T3B1Buf>();
    deserialize_generic::<T4B1Buf>();
    deserialize_generic::<T5B1Buf>();
}
