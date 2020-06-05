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

use super::{ShiftTernary, Trit, Utrit};
use std::{convert::TryFrom, fmt};

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Btrit {
    NegOne = -1,
    Zero = 0,
    PlusOne = 1,
}

impl fmt::Display for Btrit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as i8)
    }
}

impl From<Btrit> for i8 {
    fn from(value: Btrit) -> Self {
        value as i8
    }
}

impl ShiftTernary for Btrit {
    type Target = Utrit;

    fn shift(self) -> Self::Target {
        use Btrit::*;
        match self {
            NegOne => Self::Target::Zero,
            Zero => Self::Target::One,
            PlusOne => Self::Target::Two,
        }
    }
}

impl Trit for Btrit {
    fn checked_increment(self) -> Option<Self> {
        match self {
            Btrit::NegOne => Some(Btrit::Zero),
            Btrit::Zero => Some(Btrit::PlusOne),
            Btrit::PlusOne => None,
        }
    }

    fn zero() -> Self {
        Self::Zero
    }

    fn as_arbitrary_ref<'a>(&self) -> &'a Self {
        static NEG_ONE: Btrit = Btrit::NegOne;
        static ZERO: Btrit = Btrit::Zero;
        static PLUS_ONE: Btrit = Btrit::PlusOne;

        match self {
            Btrit::NegOne => &NEG_ONE,
            Btrit::Zero => &ZERO,
            Btrit::PlusOne => &PLUS_ONE,
        }
    }
}

impl TryFrom<i8> for Btrit {
    type Error = ();

    fn try_from(x: i8) -> Result<Self, Self::Error> {
        let converted = match x {
            -1 => Btrit::NegOne,
            0 => Btrit::Zero,
            1 => Btrit::PlusOne,
            _ => return Err(()),
        };
        Ok(converted)
    }
}

impl TryFrom<u8> for Btrit {
    type Error = ();

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        let converted = match x {
            0 => Btrit::Zero,
            1 => Btrit::PlusOne,
            _ => return Err(()),
        };
        Ok(converted)
    }
}
