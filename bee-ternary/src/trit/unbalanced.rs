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

use super::{Btrit, ShiftTernary, Trit};

use std::{convert::TryFrom, fmt};

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Utrit {
    Zero = 0,
    One = 1,
    Two = 2,
}

use Utrit::*;

impl fmt::Display for Utrit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as i8)
    }
}

impl From<Utrit> for i8 {
    fn from(value: Utrit) -> Self {
        value as i8
    }
}

impl ShiftTernary for Utrit {
    type Target = Btrit;

    fn shift(self) -> Self::Target {
        match self {
            Zero => Self::Target::NegOne,
            One => Self::Target::Zero,
            Two => Self::Target::PlusOne,
        }
    }
}

impl Trit for Utrit {
    fn checked_increment(self) -> Option<Self> {
        match self {
            Zero => Some(One),
            One => Some(Two),
            Two => None,
        }
    }

    fn zero() -> Self {
        Self::Zero
    }

    fn as_arbitrary_ref<'a>(&self) -> &'a Self {
        static ZERO: Utrit = Utrit::Zero;
        static ONE: Utrit = Utrit::One;
        static TWO: Utrit = Utrit::Two;

        match self {
            Utrit::Zero => &ZERO,
            Utrit::One => &ONE,
            Utrit::Two => &TWO,
        }
    }
}

impl Utrit {
    pub(crate) fn from_u8(x: u8) -> Self {
        match x {
            0 => Zero,
            1 => One,
            2 => Two,
            x => panic!("Invalid trit representation '{}'", x),
        }
    }

    pub(crate) fn into_u8(self) -> u8 {
        match self {
            Zero => 0,
            One => 1,
            Two => 2,
        }
    }
}

impl TryFrom<i8> for Utrit {
    type Error = ();

    fn try_from(x: i8) -> Result<Self, Self::Error> {
        let converted = match x {
            0 => Zero,
            1 => One,
            2 => Two,
            _ => Err(())?,
        };
        Ok(converted)
    }
}

impl TryFrom<u8> for Utrit {
    type Error = ();

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        let converted = match x {
            0 => Zero,
            1 => One,
            2 => Two,
            _ => Err(())?,
        };
        Ok(converted)
    }
}
