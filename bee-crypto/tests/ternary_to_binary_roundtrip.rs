// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(deprecated)]

use bee_crypto::ternary::bigint::{binary_representation::U32Repr, endianness::LittleEndian, I384, T242, T243, U384};
use bee_ternary::{Btrit, Utrit};

#[test]
fn t243_max_exceeds_u384_range() {
    let t243_max = T243::<Btrit>::max();
    let error = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243_max);
    assert!(error.is_err());
}

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
