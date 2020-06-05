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

mod common;

use bee_ternary::*;
use std::convert::TryFrom;

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
