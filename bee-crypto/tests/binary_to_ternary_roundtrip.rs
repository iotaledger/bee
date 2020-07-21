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

use bee_crypto::ternary::bigint::{binary_representation::U32Repr, endianness::LittleEndian, I384, T243, U384};
use bee_ternary::{Btrit, Utrit};

use std::convert::TryInto;

macro_rules! binary_to_ternary_roundtrip {
    ( ( $($binary_type:tt)* ), ( $($ternary_type:tt)* ), $testname:ident, $val_fn:ident ) => {
        #[test]
        fn $testname() {
            let original = $($binary_type)*::$val_fn();
            let ternary = $($ternary_type)*::from(original);
            let roundtripped = TryInto::<$($binary_type)*>::try_into(ternary);
            assert!(roundtripped.is_ok());
            assert_eq!(roundtripped.unwrap(), original);
        }
    };

    ( ( $( $binary_type:tt )* ),
      ( $( $ternary_type:tt )* ),
      [ $testname:ident, $val_fn:ident ]
      $(,)?
    ) => {
        binary_to_ternary_roundtrip!(( $($binary_type)* ), ( $($ternary_type)* ), $testname, $val_fn);
    };

    ( ( $( $binary_type:tt )* ),
      ( $( $ternary_type:tt )* ),
      [ $testname:ident, $val_fn:ident ],
      $( [ $tail_testname:ident, $tail_val_fn:ident ] ),+
      $(,)?
    ) => {
        binary_to_ternary_roundtrip!(
            ( $($binary_type)* ),
            ( $($ternary_type)* ),
            [ $testname, $val_fn ]
        );
        binary_to_ternary_roundtrip!(( $($binary_type)* ), ( $($ternary_type)* ), $( [$tail_testname, $tail_val_fn] ),+);
    };

    ( $modname:ident,
      ( $( $binary_type:tt )* ),
      ( $( $ternary_type:tt )* ),
      $( [ $testname:ident, $val_fn:ident ] ),+
      $(,)?
    ) => {
        mod $modname {
            use super::*;
            binary_to_ternary_roundtrip!(( $($binary_type)* ), ( $($ternary_type)* ), $( [$testname, $val_fn] ),+);
        }
    };
}

binary_to_ternary_roundtrip!(
    u384,
    (U384::<LittleEndian, U32Repr>),
    (T243::<Utrit>),
    [zero_to_self, zero],
    [one_to_self, one],
    [max_to_self, max],
);

binary_to_ternary_roundtrip!(
    i384,
    (I384::<LittleEndian, U32Repr>),
    (T243::<Btrit>),
    [zero_to_self, zero],
    [one_to_self, one],
    [two_to_self, two],
    [neg_one_to_self, neg_one],
    [neg_two_to_self, neg_two],
    [max_to_self, max],
    [min_to_self, min],
);
