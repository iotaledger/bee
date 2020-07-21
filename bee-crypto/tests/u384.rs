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

#[macro_use]
mod test_macros;

use bee_crypto::ternary::bigint::u384::{LE_U32_0, LE_U32_1, LE_U32_2, LE_U32_HALF_MAX, LE_U32_MAX};

#[test]
fn half_max_plus_half_max_is_max_minus_1() {
    let mut left = *LE_U32_HALF_MAX;
    left.add_inplace(*LE_U32_HALF_MAX);
    let mut right = LE_U32_MAX;
    right.sub_inplace(LE_U32_1);
    assert_eq!(left, right);
}

test_binary_op!(
    [zero_plus_one_is_one, add_inplace, LE_U32_0, LE_U32_1, LE_U32_1],
    [zero_plus_two_is_two, add_inplace, LE_U32_0, LE_U32_2, LE_U32_2],
    [zero_minus_one_is_max, sub_inplace, LE_U32_0, LE_U32_1, LE_U32_MAX],
    [zero_plus_max_is_max, add_inplace, LE_U32_0, LE_U32_MAX, LE_U32_MAX],
    [one_minus_one_is_zero, sub_inplace, LE_U32_1, LE_U32_1, LE_U32_0],
    [one_minus_two_is_max, sub_inplace, LE_U32_1, LE_U32_2, LE_U32_MAX],
    [one_plus_one_is_two, add_inplace, LE_U32_1, LE_U32_1, LE_U32_2],
    [max_minus_max_is_zero, sub_inplace, LE_U32_MAX, LE_U32_MAX, LE_U32_0],
    [max_minus_zero_is_max, sub_inplace, LE_U32_MAX, LE_U32_0, LE_U32_MAX],
    [max_plus_zero_is_max, add_inplace, LE_U32_MAX, LE_U32_0, LE_U32_MAX],
    [max_plus_one_is_zero, add_inplace, LE_U32_MAX, LE_U32_1, LE_U32_0],
    [max_plus_two_is_one, add_inplace, LE_U32_MAX, LE_U32_2, LE_U32_1],
);

test_binary_op_calc_result!([
    max_plus_max_is_max_minus_one,
    add_inplace,
    LE_U32_MAX,
    LE_U32_MAX,
    sub_inplace,
    LE_U32_MAX,
    LE_U32_1
],);
