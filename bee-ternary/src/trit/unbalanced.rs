// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{Btrit, ShiftTernary, Trit};
use crate::convert;
use num_traits::{CheckedAdd, CheckedSub, Num};
use std::{convert::TryFrom, fmt};

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i8)]
#[allow(missing_docs)]
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

    fn add_to_num<I: Num + CheckedAdd + CheckedSub>(&self, n: I) -> Result<I, convert::Error> {
        match self {
            Utrit::Zero => Ok(n),
            Utrit::One => n.checked_add(&I::one()).ok_or(convert::Error::Overflow),
            Utrit::Two => n.checked_add(&(I::one() + I::one())).ok_or(convert::Error::Overflow),
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
            _ => return Err(()),
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
            _ => return Err(()),
        };
        Ok(converted)
    }
}
