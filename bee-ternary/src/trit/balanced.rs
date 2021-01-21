// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{ShiftTernary, Trit, Utrit};
use crate::convert;
use num_traits::{CheckedAdd, CheckedSub, Num};
use std::{convert::TryFrom, fmt, ops::Neg};

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i8)]
#[allow(missing_docs)]
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

impl Neg for Btrit {
    type Output = Self;

    fn neg(self) -> Self {
        use Btrit::*;
        match self {
            NegOne => PlusOne,
            Zero => Zero,
            PlusOne => NegOne,
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

    fn add_to_num<I: Num + CheckedAdd + CheckedSub>(&self, n: I) -> Result<I, convert::Error> {
        match self {
            Btrit::NegOne => n.checked_sub(&I::one()).ok_or(convert::Error::Underflow),
            Btrit::Zero => Ok(n),
            Btrit::PlusOne => n.checked_add(&I::one()).ok_or(convert::Error::Overflow),
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
