// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_crypto::ternary::bigint::{binary_representation::U32Repr, endianness::LittleEndian, I384, U384};

#[test]
fn shift_i384_min_is_u384_zero() {
    let min_i384 = I384::<LittleEndian, U32Repr>::min();
    let zero_u384 = min_i384.shift_into_u384();
    assert_eq!(zero_u384, U384::<LittleEndian, U32Repr>::zero());
}

#[test]
fn shift_u384_max_is_i384_max() {
    let max_u384 = U384::<LittleEndian, U32Repr>::max();
    let max_i384 = max_u384.shift_into_i384();
    assert_eq!(max_i384, I384::<LittleEndian, U32Repr>::max());
}

#[test]
fn shift_i384_max_is_u384_max() {
    let max_i384 = I384::<LittleEndian, U32Repr>::max();
    let max_u384 = max_i384.shift_into_u384();
    assert_eq!(max_u384, U384::<LittleEndian, U32Repr>::max());
}
