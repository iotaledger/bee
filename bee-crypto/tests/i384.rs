// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(deprecated)]

#[macro_use]
mod test_macros;

use bee_crypto::ternary::bigint::i384::{
    LE_U32_0, LE_U32_1, LE_U32_2, LE_U32_MAX, LE_U32_MIN, LE_U32_NEG_1, LE_U32_NEG_2,
};

test_binary_op!(
    [min_minus_one_is_max, sub_inplace, LE_U32_MIN, LE_U32_1, LE_U32_MAX],
    [
        min_plus_neg_one_is_max,
        add_inplace,
        LE_U32_MIN,
        LE_U32_NEG_1,
        LE_U32_MAX
    ],
    [min_minus_zero_is_min, sub_inplace, LE_U32_MIN, LE_U32_0, LE_U32_MIN],
    [min_plus_zero_is_min, add_inplace, LE_U32_MIN, LE_U32_0, LE_U32_MIN],
    [
        neg_one_minus_one_is_neg_two,
        sub_inplace,
        LE_U32_NEG_1,
        LE_U32_1,
        LE_U32_NEG_2
    ],
    [
        neg_one_minus_neg_one_is_zero,
        sub_inplace,
        LE_U32_NEG_1,
        LE_U32_NEG_1,
        LE_U32_0
    ],
    [neg_one_plus_one_is_zero, add_inplace, LE_U32_NEG_1, LE_U32_1, LE_U32_0],
    [
        neg_one_plus_neg_one_is_neg_two,
        add_inplace,
        LE_U32_NEG_1,
        LE_U32_NEG_1,
        LE_U32_NEG_2
    ],
    [zero_minus_one_is_neg_one, sub_inplace, LE_U32_0, LE_U32_1, LE_U32_NEG_1],
    [zero_minus_neg_one_is_one, sub_inplace, LE_U32_0, LE_U32_NEG_1, LE_U32_1],
    [zero_plus_one_is_one, add_inplace, LE_U32_0, LE_U32_1, LE_U32_1],
    [
        zero_plus_neg_one_is_neg_one,
        add_inplace,
        LE_U32_0,
        LE_U32_NEG_1,
        LE_U32_NEG_1
    ],
    [one_minus_neg_one_is_two, sub_inplace, LE_U32_1, LE_U32_NEG_1, LE_U32_2],
    [one_minus_one_is_zero, sub_inplace, LE_U32_1, LE_U32_1, LE_U32_0],
    [one_plus_one_is_two, add_inplace, LE_U32_1, LE_U32_1, LE_U32_2],
    [one_plus_neg_one_is_zero, add_inplace, LE_U32_1, LE_U32_NEG_1, LE_U32_0],
    [max_plus_one_is_min, add_inplace, LE_U32_MAX, LE_U32_1, LE_U32_MIN],
    [
        max_minus_neg_one_is_min,
        sub_inplace,
        LE_U32_MAX,
        LE_U32_NEG_1,
        LE_U32_MIN
    ],
);

test_binary_op_calc_result!(
    [
        min_minus_two_is_max_minus_one,
        sub_inplace,
        LE_U32_MIN,
        LE_U32_2,
        sub_inplace,
        LE_U32_MAX,
        LE_U32_1
    ],
    [
        min_plus_one_is_max_plus_two,
        add_inplace,
        LE_U32_MIN,
        LE_U32_1,
        add_inplace,
        LE_U32_MAX,
        LE_U32_2
    ],
);

test_endianness_toggle!((I384), [u8_repr, U8Repr], [u32_repr, U32Repr],);

test_endianness_roundtrip!((I384), [u8_repr, U8Repr], [u32_repr, U32Repr],);

test_repr_roundtrip!((I384), [big_endian, BigEndian], [little_endian, LittleEndian],);
