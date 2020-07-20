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

use bee_crypto::ternary::bigint::{binary_representation::U32Repr, endianness::LittleEndian, I384, T242, T243, U384};
use bee_ternary::{Btrit, Utrit};

use std::convert::TryInto;

#[test]
fn t243_max_exceeds_u384_range() {
    let t243_max = T243::<Btrit>::max();
    let error = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243_max);
    assert!(error.is_err());
}

// #[test]
// fn t243_min_exceeds_u384_range() {
//     let t243_min = T243::min();
//     let error = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243_min);
//     assert!(error.is_err());
// }

// #[test]
// fn u384_max_in_t243_is_u384_max_in_t243() {
//     let converted = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243::I384_MAX.clone());
//     assert!(converted.is_ok());
//     let roundtripped: T243 = converted.unwrap().into();
//     assert_eq!(roundtripped, *t243::I384_MAX);
// }

// #[test]
// fn u384_min_in_t243_is_u384_min_in_t243() {
//     let converted = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243::I384_MIN.clone());
//     assert!(converted.is_ok());
//     let roundtripped: T243 = converted.unwrap().into();
//     assert_eq!(roundtripped, *t243::I384_MIN);
// }

macro_rules! ternary_roundtrip {
    ( @basecase: ($($ternary_type:tt)*), ($($binary_type:tt)*), $testname:ident, $val_fn:ident ) => {
        #[test]
        fn $testname() {
            let original = $($ternary_type)*::$val_fn();
            let converted = Into::<$($binary_type)*>::into(original.clone());
            let roundtripped = TryInto::<$($ternary_type)*>::try_into(converted);
            assert!(roundtripped.is_ok());
            assert_eq!(roundtripped.unwrap(), original);
        }
    };

    ( ( $($ternary_type:tt)* )
      <>
      ( $($binary_type:tt)* )
      =>
      [$testname:ident, $val_fn:ident] $(,)?
    ) => {
        ternary_roundtrip!(@basecase: ($($ternary_type)*), ($($binary_type)*), $testname, $val_fn);
    };

    ( ( $($ternary_type:tt)* )
      <>
      ( $($binary_type:tt)* )
      =>
      [$testname:ident, $val_fn:ident],
      $( [$rest_testname:ident, $rest_val_fn:ident] ),+ $(,)?
    ) => {
        ternary_roundtrip!( ( $($ternary_type)* ) <> ( $($binary_type)* ) => [$testname, $val_fn] );
        ternary_roundtrip!(
            ($($ternary_type)*) <> ($($binary_type)*)
            =>
            $([$rest_testname, $rest_val_fn]),+);
    };

    ( $modname:ident:
      ( $($ternary_type:tt)* )
      <>
      ( $($binary_type:tt)* )
      =>
      $( [$testname:ident, $val_fn:ident] ),+ $(,)?
    ) => {
        mod $modname {
            use super::*;

            ternary_roundtrip!(
              ( $($ternary_type)* )
              <>
              ( $($binary_type)* )
              =>
              $( [$testname, $val_fn] ),+
            );
        }
    };
}

ternary_roundtrip!(
    t242_btrit: (T242::<Btrit>) <> (I384<LittleEndian, U32Repr>)
    =>
    [zero_is_zero, zero],
    [one_is_one, one],
    [neg_one_is_neg_one, neg_one],
    [max_is_max, max],
);

ternary_roundtrip!(
    t242_utrit: (T242::<Utrit>) <> (U384<LittleEndian, U32Repr>)
    =>
    [zero_is_zero, zero],
    [one_is_one, one],
    [two_is_two, two],
    [max_is_max, max],
);
