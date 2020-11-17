// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
