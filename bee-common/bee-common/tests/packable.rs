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

use bee_common::packable::Packable;

macro_rules! impl_packable_test_for_num {
    ($name:ident, $ty:ident, $value:expr) => {
        #[test]
        fn $name() {
            let num: $ty = $value;
            assert_eq!(num, $ty::unpack(&mut num.pack_new().unwrap().as_slice()).unwrap());
        }
    };
}

impl_packable_test_for_num!(pack_unpack_i8, i8, 0x6F);
impl_packable_test_for_num!(pack_unpack_u8, u8, 0x6F);
impl_packable_test_for_num!(pack_unpack_i16, i16, 0x6F7B);
impl_packable_test_for_num!(pack_unpack_u16, u16, 0x6F7B);
impl_packable_test_for_num!(pack_unpack_i32, i32, 0x6F7BD423);
impl_packable_test_for_num!(pack_unpack_u32, u32, 0x6F7BD423);
impl_packable_test_for_num!(pack_unpack_i64, i64, 0x6F7BD423100423DB);
impl_packable_test_for_num!(pack_unpack_u64, u64, 0x6F7BD423100423DB);
impl_packable_test_for_num!(pack_unpack_i128, i128, 0x6F7BD423100423DBFF127B91CA0AB123);
impl_packable_test_for_num!(pack_unpack_u128, u128, 0x6F7BD423100423DBFF127B91CA0AB123);
