// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_macros)]
macro_rules! test_binary_op {
    ( $( [$testname:ident, $binop:ident, $fst:ident, $snd:ident, $res:ident] ),+ $(,)? ) => {
        mod little_endian_binary_op {
            use super::*;

            $(
                #[test]
                fn $testname() {
                    let mut fst = $fst;
                    fst.$binop($snd);
                    assert_eq!(fst, $res);
                }
            )+
        }
    };
}

#[allow(unused_macros)]
macro_rules! test_binary_op_calc_result {
    ( $( [
         $testname:ident,
         $lft_binop:ident, $lft_fst:ident, $lft_snd:ident,
         $rgt_binop:ident, $rgt_fst:ident, $rgt_snd:ident
        ]
      ),+ $(,)?
    ) => {
        mod little_endian_binary_op_calc_result {
            use super::*;

            $(
                #[test]
                fn $testname() {
                    let mut left = $lft_fst;
                    left.$lft_binop($lft_snd);
                    let mut right = $rgt_fst;
                    right.$rgt_binop($rgt_snd);
                    assert_eq!(left, right);
                }
            )+
        }
    };
}

#[allow(unused_macros)]
macro_rules! endianness_roundtrip_test_function {
    ( $testname:ident, $repr:ty, $src_endian:ty, $dst_endian:ty, $val_fn:ident ) => {
        #[test]
        fn $testname() {
            let original = Root::<$src_endian, $repr>::$val_fn();
            let converted = Into::<Root<$dst_endian, $repr>>::into(original);
            let roundtripped = Into::<Root<$src_endian, $repr>>::into(converted);
            assert_eq!(roundtripped, original);
        }
    };
}

#[allow(unused_macros)]
macro_rules! test_endianness_roundtrip {
    ( $modname:ident, $repr:ty, $src_endian:ty, $dst_endian:ty) => {
        mod $modname {
            use super::*;
            endianness_roundtrip_test_function!(zero_is_zero, $repr, $src_endian, $dst_endian, zero);
            endianness_roundtrip_test_function!(one_is_one, $repr, $src_endian, $dst_endian, one);
            endianness_roundtrip_test_function!(neg_one_is_neg_one, $repr, $src_endian, $dst_endian, neg_one);
            endianness_roundtrip_test_function!(two_is_two, $repr, $src_endian, $dst_endian, two);
            endianness_roundtrip_test_function!(neg_two_is_neg_two, $repr, $src_endian, $dst_endian, neg_two);
            endianness_roundtrip_test_function!(max_is_max, $repr, $src_endian, $dst_endian, max);
            endianness_roundtrip_test_function!(min_is_min, $repr, $src_endian, $dst_endian, min);
        }
    };

    (
        ( $($root:tt)* ),
        $( [ $modname:ident, $repr:ty ] ),+
        $(,)?
    ) => {

        mod endianness_roundtrip {
            use bee_crypto::ternary::bigint::$($root)* as Root;
            use bee_crypto::ternary::bigint::{
                binary_representation::{
                    U8Repr,
                    U32Repr,
                },
                endianness::{
                    BigEndian,
                    LittleEndian,
                }
            };
            $(
                mod $modname {
                    use super::*;
                    test_endianness_roundtrip!(big, $repr, BigEndian, LittleEndian);
                    test_endianness_roundtrip!(little, $repr, LittleEndian, BigEndian);
                }
            )+
        }
    };
}

#[allow(unused_macros)]
macro_rules! endianness_toggle_test_function {
    ( $testname:ident, $repr:ty, $src_endian:ty, $dst_endian:ty, $val_fn:ident) => {
        #[test]
        fn $testname() {
            let original = Root::<$src_endian, $repr>::$val_fn();
            let target = Root::<$dst_endian, $repr>::$val_fn();
            let converted = Into::<Root<$dst_endian, $repr>>::into(original);
            assert_eq!(converted, target);
        }
    };
}

#[allow(unused_macros)]
macro_rules! test_endianness_toggle {
    ( $modname:ident, $repr:ty, $src_endian:ty, $dst_endian:ty ) => {
        mod $modname {
            use super::*;
            endianness_toggle_test_function!(zero_is_zero, $repr, $src_endian, $dst_endian, zero);
            endianness_toggle_test_function!(one_is_one, $repr, $src_endian, $dst_endian, one);
            endianness_toggle_test_function!(neg_one_is_neg_one, $repr, $src_endian, $dst_endian, neg_one);
            endianness_toggle_test_function!(two_is_two, $repr, $src_endian, $dst_endian, two);
            endianness_toggle_test_function!(neg_two_is_neg_two, $repr, $src_endian, $dst_endian, neg_two);
            endianness_toggle_test_function!(max_is_max, $repr, $src_endian, $dst_endian, max);
            endianness_toggle_test_function!(min_is_min, $repr, $src_endian, $dst_endian, min);
        }
    };

    ( $modname:ident, $repr:ty, $src_endian:ty, $dst_endian:ty ) => {
        mod $modname {
            use super::*;
            endianness_toggle_test_function!(zero_is_zero, $repr, $src_endian, $dst_endian, zero);
            endianness_toggle_test_function!(one_is_one, $repr, $src_endian, $dst_endian, one);
            endianness_toggle_test_function!(neg_one_is_neg_one, $repr, $src_endian, $dst_endian, neg_one);
            endianness_toggle_test_function!(two_is_two, $repr, $src_endian, $dst_endian, two);
            endianness_toggle_test_function!(neg_two_is_neg_two, $repr, $src_endian, $dst_endian, neg_two);
            endianness_toggle_test_function!(max_is_max, $repr, $src_endian, $dst_endian, max);
            endianness_toggle_test_function!(min_is_min, $repr, $src_endian, $dst_endian, min);
        }
    };

    (
        ( $($root:tt)* ),
        $( [ $modname:ident, $repr:ty ] ),+
        $(,)?
    ) => {

        mod toggle_endianness {
            use bee_crypto::ternary::bigint::$($root)* as Root;
            use bee_crypto::ternary::bigint::{
                binary_representation::{
                    U8Repr,
                    U32Repr,
                },
                endianness::{
                    BigEndian,
                    LittleEndian,
                }
            };
            $(
                mod $modname {
                    use super::*;
                    test_endianness_toggle!(big_to_little, $repr, BigEndian, LittleEndian);
                    test_endianness_toggle!(little_to_big, $repr, LittleEndian, BigEndian);
                }
            )+
        }
    };
}

#[allow(unused_macros)]
macro_rules! repr_roundtrip_test_function {
    ( $testname:ident, $endianness:ty, $src_repr:ty, $dst_repr:ty, $val_fn:ident ) => {
        #[test]
        fn $testname() {
            let original = Root::<$endianness, $src_repr>::$val_fn();
            let converted = Into::<Root<$endianness, $dst_repr>>::into(original);
            let roundtripped = Into::<Root<$endianness, $src_repr>>::into(converted);
            assert_eq!(roundtripped, original);
        }
    };
}

#[allow(unused_macros)]
macro_rules! test_repr_roundtrip {
    ( $modname:ident, $endianness:ty, $src_repr:ty, $dst_repr:ty ) => {
        mod $modname {
            use super::*;
            repr_roundtrip_test_function!(zero_is_zero, $endianness, $src_repr, $dst_repr, zero);
            repr_roundtrip_test_function!(one_is_one, $endianness, $src_repr, $dst_repr, one);
            repr_roundtrip_test_function!(neg_one_is_neg_one, $endianness, $src_repr, $dst_repr, neg_one);
            repr_roundtrip_test_function!(two_is_two, $endianness, $src_repr, $dst_repr, two);
            repr_roundtrip_test_function!(neg_two_is_neg_two, $endianness, $src_repr, $dst_repr, neg_two);
            repr_roundtrip_test_function!(max_is_max, $endianness, $src_repr, $dst_repr, max);
            repr_roundtrip_test_function!(min_is_min, $endianness, $src_repr, $dst_repr, min);
        }
    };

    (
        ( $( $root:tt )* ),
        $( [ $modname:ident, $endianness:ty ] ),+
        $(,)?
    ) => {
        mod test_repr_roundtrip {
            use bee_crypto::ternary::bigint::$($root)* as Root;
            use bee_crypto::ternary::bigint::{
                binary_representation::{
                    U8Repr,
                    U32Repr,
                },
                endianness::{
                    BigEndian,
                    LittleEndian,
                }
            };

            $(
                mod $modname {
                    use super::*;
                    test_repr_roundtrip!(u8_repr, $endianness, U8Repr, U32Repr);
                    test_repr_roundtrip!(u32_repr, $endianness, U32Repr, U8Repr);
                }
            )+
        }
    };
}
