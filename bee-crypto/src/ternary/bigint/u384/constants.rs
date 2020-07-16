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

use crate::ternary::bigint::{
    binary_representation::{U32Repr, U8Repr},
    endianness::{BigEndian, LittleEndian},
    T242, T243,
};

use super::U384;

use bee_ternary::Utrit;

use lazy_static::lazy_static;

use std::convert::TryFrom;

lazy_static! {
    /// U384 big-endian `u32` represented half of maximum value.
    pub static ref BE_U32_HALF_MAX: U384<BigEndian, U32Repr> = (*LE_U32_HALF_MAX).into();
    /// U384 big-endian `u32` represented half of T242 maximum value.
    pub static ref BE_U32_HALF_MAX_T242: U384<BigEndian, U32Repr> =
        Into::<U384<BigEndian, U32Repr>>::into(*LE_U32_HALF_MAX_T242);
    /// U384 little-endian `u32` represented half of maximum value.
    pub static ref LE_U32_HALF_MAX: U384<LittleEndian, U32Repr> = {
        let mut u384_max = U384::<LittleEndian, U32Repr>::max();
        u384_max.divide_by_two();
        u384_max
    };
    /// U384 little-endian `u32` represented half of T242 maximum value.
    pub static ref LE_U32_HALF_MAX_T242: U384<LittleEndian, U32Repr> = {
        T242::half_max().into()
    };
    /// U384 little-endian `u32` represented -half of T242 maximum value.
    pub static ref LE_U32_NEG_HALF_MAX_T242: U384<LittleEndian, U32Repr> = {
        let mut bigint = U384::<LittleEndian, U32Repr>::zero();
        bigint.sub_inplace(*LE_U32_HALF_MAX_T242);
        bigint
    };
    /// U384 little-endian `u32` represented T243 with only last trit set to 1.
    pub static ref LE_U32_ONLY_T243_OCCUPIED: U384<LittleEndian, U32Repr> = {
        let mut t243 = T243::<Utrit>::zero();
        t243.set(242, Utrit::One);
        // Safe because we know this value is in the U384 space.
        U384::<LittleEndian, U32Repr>::try_from(t243).unwrap()
    };
    /// U384 little-endian `u32` represented T242 maximum value.
    pub static ref LE_U32_MAX_T242: U384<LittleEndian, U32Repr> = {
        T242::<Utrit>::max().into()
    };
}

/// U384 big-endian `u8` represented 0.
pub const BE_U8_0: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

/// U384 big-endian `u8` represented 1.
pub const BE_U8_1: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
]);

/// U384 big-endian `u8` represented 2.
pub const BE_U8_2: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
]);

/// U384 big-endian `u8` represented maximum value.
pub const BE_U8_MAX: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
]);

/// U384 big-endian `u32` represented 0.
pub const BE_U32_0: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

/// U384 big-endian `u32` represented 1.
pub const BE_U32_1: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0001,
]);

/// U384 big-endian `u32` represented 2.
pub const BE_U32_2: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
]);

/// U384 big-endian `u32` represented maximum value.
pub const BE_U32_MAX: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
]);

/// U384 little-endian `u8` represented 0.
pub const LE_U8_0: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

/// U384 little-endian `u8` represented 1.
pub const LE_U8_1: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

/// U384 little-endian `u8` represented 2.
pub const LE_U8_2: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

/// U384 little-endian `u8` represented maximum value.
pub const LE_U8_MAX: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
]);

/// U384 little-endian `u32` represented 0.
pub const LE_U32_0: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

/// U384 little-endian `u32` represented 1.
pub const LE_U32_1: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

/// U384 little-endian `u32` represented 2.
pub const LE_U32_2: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0x0000_0002,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

/// U384 little-endian `u32` represented maximum value.
pub const LE_U32_MAX: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
]);
