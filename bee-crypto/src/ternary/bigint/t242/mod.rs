// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This module contains logic to convert an integer encoded by 242 trits to the same integer encoded by 384 bits (or 48
//! signed bytes, `i8`).
//!
//! At the core of this a slice of binary-coded, balanced trits is interpreted fanned-out `t242`, where `t242` is used
//! analogous to `i64` or `u64`. If the latter are 64-bit signed/unsigned integer types, then `t242` is a 242-trit
//! integer type. Analogous to fanning out a `u64` into 64 individual bits, `t242` is fanned out into 242 trits, each
//! (rather inefficiently) represented by one `u8`.

mod constants;

pub use constants::{
    BTRIT_0, BTRIT_1, BTRIT_MAX, BTRIT_MIN, BTRIT_NEG_1, UTRIT_0, UTRIT_1, UTRIT_2, UTRIT_U384_MAX, UTRIT_U384_MAX_HALF,
};

use crate::ternary::bigint::{
    binary_representation::{U32Repr, U8Repr},
    endianness::{BigEndian, LittleEndian},
    error::Error,
    u384, I384, T243, U384,
};

use bee_ternary::{Btrit, ShiftTernary, T1B1Buf, Trit, TritBuf, Utrit};

use std::cmp::Ordering;

def_and_impl_ternary!(T242, 242);

impl<T: Trit> T242<T> {
    /// Converts the `T242` to a `T243`.
    pub fn into_t243(self) -> T243<T> {
        let mut trit_buf = self.into_inner();
        trit_buf.push(T::zero());
        T243::new(trit_buf)
    }
}

impl T242<Btrit> {
    /// Converts a big-endian `u8` represented `I384` to a balanced `T242` while ignoring its MST.
    pub fn from_i384_ignoring_mst(value: I384<BigEndian, U8Repr>) -> Self {
        let value: I384<LittleEndian, U8Repr> = value.into();
        let mut value: I384<LittleEndian, U32Repr> = value.into();

        value.zero_most_significant_trit();

        let mut unsigned_binary = value.as_u384();
        unsigned_binary.add_inplace(*u384::LE_U32_HALF_MAX_T242);

        let t243_utrit: T243<Utrit> = unsigned_binary.into();
        let t243_btrit = t243_utrit.into_shifted();
        t243_btrit.into_t242()
    }

    /// Tries to convert a big-endian `u32` represented `I384` to a balanced `T242`.
    pub fn try_from_i384(value: I384<LittleEndian, U32Repr>) -> Result<Self, Error> {
        let mut unsigned_binary = value.as_u384();
        unsigned_binary.add_inplace(*u384::LE_U32_HALF_MAX_T242);
        if unsigned_binary > *u384::LE_U32_MAX_T242 {
            return Err(Error::BinaryExceedsTernaryRange);
        }
        let unsigned_ternary: T243<Utrit> = unsigned_binary.into();
        let signed_ternary = unsigned_ternary.into_shifted();
        Ok(signed_ternary.into_t242())
    }
}

impl T242<Utrit> {
    /// Converts a big-endian `u8` represented `U384` to an unbalanced `T242` while ignoring its MSB.
    pub fn from_u384_be_u8repr_ignoring_msd(value: U384<BigEndian, U8Repr>) -> Self {
        let value: U384<LittleEndian, U8Repr> = value.into();
        let value: U384<LittleEndian, U32Repr> = value.into();
        let t243_utrit: T243<Utrit> = value.into();
        t243_utrit.into_t242()
    }

    /// Tries to convert a little-endian `u32` represented `I384` to an unbalanced `T242`.
    pub fn try_from_i384(value: I384<LittleEndian, U32Repr>) -> Result<Self, Error> {
        let mut unsigned_binary = value.as_u384();
        unsigned_binary.add_inplace(*u384::LE_U32_HALF_MAX_T242);
        if unsigned_binary > *u384::LE_U32_MAX_T242 {
            return Err(Error::BinaryExceedsTernaryRange);
        }
        let unsigned_ternary: T243<Utrit> = unsigned_binary.into();
        Ok(unsigned_ternary.into_t242())
    }
}

impl TryFrom<I384<BigEndian, U8Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<BigEndian, U8Repr>) -> Result<Self, Self::Error> {
        let as_littleendian: I384<LittleEndian, U8Repr> = value.into();
        let as_littleendian_u32repr: I384<LittleEndian, U32Repr> = as_littleendian.into();
        as_littleendian_u32repr.try_into()
    }
}

impl TryFrom<I384<BigEndian, U32Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<BigEndian, U32Repr>) -> Result<Self, Self::Error> {
        let as_littleendian: I384<LittleEndian, U32Repr> = value.into();
        as_littleendian.try_into()
    }
}

impl TryFrom<I384<LittleEndian, U8Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<LittleEndian, U8Repr>) -> Result<Self, Self::Error> {
        let as_u32repr: I384<LittleEndian, U32Repr> = value.into();
        as_u32repr.try_into()
    }
}

impl TryFrom<I384<LittleEndian, U32Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<LittleEndian, U32Repr>) -> Result<Self, Self::Error> {
        Self::try_from_i384(value)
    }
}

impl From<U384<LittleEndian, U32Repr>> for T242<Utrit> {
    fn from(binary_value: U384<LittleEndian, U32Repr>) -> Self {
        let ternary_value: T243<Utrit> = binary_value.into();
        ternary_value.into_t242()
    }
}
