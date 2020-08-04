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

use crate::bundled::constants::{
    ADDRESS, ADDRESS_TRIT_LEN, NONCE, NONCE_TRIT_LEN, PAYLOAD, PAYLOAD_TRIT_LEN, TAG, TAG_TRIT_LEN,
};

use bee_crypto::ternary::Hash;
use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};

use std::cmp::PartialEq;

#[derive(Debug)]
pub enum BundledTransactionFieldError {
    FieldWrongLength,
}

pub trait BundledTransactionField: Sized + BundledTransactionFieldType {
    type Inner: ToOwned + ?Sized;
    fn try_from_inner(buffer: <Self::Inner as ToOwned>::Owned) -> Result<Self, BundledTransactionFieldError>;
    fn from_inner_unchecked(buffer: <Self::Inner as ToOwned>::Owned) -> Self;

    fn to_inner(&self) -> &Self::Inner;

    fn trit_len() -> usize;
}

pub trait NumTritsOfValue {
    fn num_trits(&self) -> usize;
}

pub trait BundledTransactionFieldType {
    type InnerType: NumTritsOfValue + ?Sized;

    fn is_trits_type() -> bool;
}

impl NumTritsOfValue for Trits {
    fn num_trits(&self) -> usize {
        self.len()
    }
}

impl NumTritsOfValue for i64 {
    fn num_trits(&self) -> usize {
        unimplemented!();
    }
}

impl NumTritsOfValue for u64 {
    fn num_trits(&self) -> usize {
        unimplemented!();
    }
}

impl NumTritsOfValue for usize {
    fn num_trits(&self) -> usize {
        unimplemented!();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Payload(pub(crate) TritBuf<T1B1Buf>);

impl Payload {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(PAYLOAD.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        PAYLOAD_TRIT_LEN
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Address(pub(crate) TritBuf<T1B1Buf>);

impl Address {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(ADDRESS.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        ADDRESS_TRIT_LEN
    }
}

impl Eq for Address {}

#[derive(Clone, Debug, PartialEq)]
pub struct Value(pub(crate) i64);

impl Value {
    pub fn trit_len() -> usize {
        unimplemented!();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tag(pub(crate) TritBuf<T1B1Buf>);

impl Tag {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(TAG.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        TAG_TRIT_LEN
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Timestamp(pub(crate) u64);

impl Timestamp {
    pub fn trit_len() -> usize {
        unimplemented!();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Index(pub(crate) usize);

impl Index {
    pub fn trit_len() -> usize {
        unimplemented!();
    }
}

impl BundledTransactionFieldType for Hash {
    type InnerType = Trits<T1B1>; // TritBuf<T1B1Buf>;

    fn is_trits_type() -> bool {
        true
    }
}

impl BundledTransactionField for Hash {
    // TODO why Trits and not TritBuf ?
    type Inner = Trits<T1B1>;

    fn to_inner(&self) -> &Self::Inner {
        self.as_trits()
    }

    fn trit_len() -> usize {
        243
    }

    fn try_from_inner(buf: <Self::Inner as ToOwned>::Owned) -> Result<Self, BundledTransactionFieldError> {
        if buf.len() != Self::trit_len() {
            return Err(BundledTransactionFieldError::FieldWrongLength);
        }

        Ok(Self::from_inner_unchecked(buf))
    }

    fn from_inner_unchecked(buf: <Self::Inner as ToOwned>::Owned) -> Self {
        let mut hash = Hash::zeros();
        (*hash).copy_from(&buf);

        hash
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Nonce(pub(crate) TritBuf<T1B1Buf>);

impl Nonce {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(NONCE.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        NONCE_TRIT_LEN
    }
}

macro_rules! impl_transaction_field {
    ( $($field_name:ident),+ $(,)?) => {
        $(
            impl BundledTransactionField for $field_name {
                type Inner = <$field_name as BundledTransactionFieldType>::InnerType;

                fn to_inner(&self) -> &Self::Inner {
                    &self.0
                }

                fn try_from_inner(val: <Self::Inner as ToOwned>::Owned) -> Result<Self, BundledTransactionFieldError> {
                    if $field_name::is_trits_type() && val.num_trits() != $field_name::trit_len() {
                        return Err(BundledTransactionFieldError::FieldWrongLength);
                    }
                    Ok(Self::from_inner_unchecked(val))
                }

                fn from_inner_unchecked(val: <Self::Inner as ToOwned>::Owned) -> Self {
                    Self(val)
                }

                fn trit_len() -> usize {
                   Self::trit_len()
                }
            }
        )+
    }
}

macro_rules! impl_transaction_field_type_for_tritbuf_fields {
    ( $($field_name:ident),+ $(,)?) => {
        $(
            impl BundledTransactionFieldType for $field_name {
                type InnerType = Trits<T1B1>;
                fn is_trits_type() -> bool {true}
            }
        )+
    }
}

impl BundledTransactionFieldType for Value {
    type InnerType = i64;

    fn is_trits_type() -> bool {
        false
    }
}

impl BundledTransactionFieldType for Index {
    type InnerType = usize;

    fn is_trits_type() -> bool {
        false
    }
}

impl BundledTransactionFieldType for Timestamp {
    type InnerType = u64;

    fn is_trits_type() -> bool {
        false
    }
}

impl_transaction_field_type_for_tritbuf_fields!(Payload, Address, Tag, Nonce);
impl_transaction_field!(Payload, Address, Tag, Nonce, Index, Value, Timestamp);
