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
use serde::{
    de::{Error, SeqAccess, Unexpected, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{convert::TryFrom, fmt, marker::PhantomData};

// Serialisation

impl Serialize for Btrit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i8((*self).into())
    }
}

impl Serialize for Utrit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i8((*self).into())
    }
}

impl<'a, T: RawEncoding> Serialize for &'a Trits<T>
where
    T::Trit: Serialize,
    T: serde::Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for trit in self.iter() {
            seq.serialize_element(&trit)?;
        }
        seq.end()
    }
}

impl<T: RawEncodingBuf> Serialize for TritBuf<T>
where
    <T::Slice as RawEncoding>::Trit: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for trit in self.iter() {
            seq.serialize_element(&trit)?;
        }
        seq.end()
    }
}

// Deserialisation

struct BtritVisitor;

impl<'de> Visitor<'de> for BtritVisitor {
    type Value = Btrit;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a value between -1 and 1 inclusive")
    }

    fn visit_u64<E: Error>(self, trit: u64) -> Result<Self::Value, E> {
        i8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Btrit::try_from(trit).map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Unsigned(trit), &self))
    }

    fn visit_i64<E: Error>(self, trit: i64) -> Result<Self::Value, E> {
        i8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Btrit::try_from(trit).map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Signed(trit), &self))
    }

    fn visit_u8<E: Error>(self, trit: u8) -> Result<Self::Value, E> {
        i8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Btrit::try_from(trit).map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Unsigned(trit as u64), &self))
    }

    fn visit_i8<E: Error>(self, trit: i8) -> Result<Self::Value, E> {
        Btrit::try_from(trit).map_err(|_| E::invalid_value(Unexpected::Signed(trit as i64), &self))
    }
}

impl<'de> Deserialize<'de> for Btrit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_i8(BtritVisitor)
    }
}

struct UtritVisitor;

impl<'de> Visitor<'de> for UtritVisitor {
    type Value = Utrit;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a value between 0 and 2 inclusive")
    }

    fn visit_u64<E: Error>(self, trit: u64) -> Result<Self::Value, E> {
        u8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Utrit::try_from(trit).map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Unsigned(trit), &self))
    }

    fn visit_i64<E: Error>(self, trit: i64) -> Result<Self::Value, E> {
        i8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Utrit::try_from(trit).map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Signed(trit), &self))
    }

    fn visit_u8<E: Error>(self, trit: u8) -> Result<Self::Value, E> {
        u8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Utrit::try_from(trit).map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Unsigned(trit as u64), &self))
    }

    fn visit_i8<E: Error>(self, trit: i8) -> Result<Self::Value, E> {
        Utrit::try_from(trit).map_err(|_| E::invalid_value(Unexpected::Signed(trit as i64), &self))
    }
}

impl<'de> Deserialize<'de> for Utrit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_i8(UtritVisitor)
    }
}

struct TritBufVisitor<T>(PhantomData<T>);

impl<'de, T: RawEncodingBuf> Visitor<'de> for TritBufVisitor<T>
where
    <T::Slice as RawEncoding>::Trit: Deserialize<'de>,
{
    type Value = TritBuf<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence of trits")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut buf = TritBuf::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(trit) = seq.next_element()? {
            buf.push(trit);
        }

        Ok(buf)
    }
}

impl<'de, T: RawEncodingBuf> Deserialize<'de> for TritBuf<T>
where
    <T::Slice as RawEncoding>::Trit: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(TritBufVisitor::<T>(PhantomData))
    }
}
