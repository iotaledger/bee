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

#![cfg(feature = "serde1")]

mod common;
use self::common::*;

use bee_ternary::{raw::*, *};
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
