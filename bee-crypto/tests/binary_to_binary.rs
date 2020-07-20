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
