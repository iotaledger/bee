// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::*;

#[test]
fn convert_correct() {
    assert_eq!(Utrit::try_from(0i8).unwrap(), Utrit::Zero);
    assert_eq!(Utrit::try_from(1i8).unwrap(), Utrit::One);
    assert_eq!(Utrit::try_from(2i8).unwrap(), Utrit::Two);

    assert_eq!(Into::<i8>::into(Utrit::Zero), 0i8);
    assert_eq!(Into::<i8>::into(Utrit::One), 1i8);
    assert_eq!(Into::<i8>::into(Utrit::Two), 2i8);
}

#[test]
fn convert_balanced() {
    assert_eq!(Btrit::NegOne.shift(), Utrit::Zero);
    assert_eq!(Btrit::Zero.shift(), Utrit::One);
    assert_eq!(Btrit::PlusOne.shift(), Utrit::Two);
}

#[test]
#[should_panic]
fn convert_incorrect_0() {
    Utrit::try_from(-1i8).unwrap();
}

#[test]
#[should_panic]
fn convert_incorrect_1() {
    Utrit::try_from(3i8).unwrap();
}

#[test]
#[should_panic]
fn convert_incorrect_2() {
    Btrit::try_from(-2i8).unwrap();
}

#[test]
#[should_panic]
fn convert_incorrect_3() {
    Btrit::try_from(2i8).unwrap();
}
