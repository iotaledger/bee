// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::convert;
use num_traits::{CheckedAdd, CheckedSub, Num};
use std::{convert::TryFrom, fmt, hash};

/// Balanced trits.
pub mod balanced;
/// Unbalanced trits.
pub mod unbalanced;

// Reexports
pub use self::{balanced::Btrit, unbalanced::Utrit};

/// A trait implemented by both balanced ([`Btrit`]) and unbalanced ([`Utrit`]) trits.
pub trait Trit:
    Copy + Sized + fmt::Debug + hash::Hash + Into<i8> + Ord + PartialEq + ShiftTernary + TryFrom<i8> + fmt::Display
{
    /// Attempt to increment the value of this trit, returning [`None`] if an overflow occurred.
    fn checked_increment(self) -> Option<Self>;

    /// The zero value of this trit.
    fn zero() -> Self;

    /// Turn this trit reference into one with an arbitrary lifetime.
    ///
    /// Note that this is largely an implementation detail and is rarely useful for API users.
    fn as_arbitrary_ref<'a>(&self) -> &'a Self;

    /// Attempt to add this trit to a numeric value.
    fn add_to_num<I: Num + CheckedAdd + CheckedSub>(&self, n: I) -> Result<I, convert::Error>;
}

/// A trait implemented by trits that can be shifted between balance domains.
// TODO: Is this a good API?
pub trait ShiftTernary: Sized {
    /// The trit type that results from shifting this trit.
    type Target: ShiftTernary<Target = Self>;

    /// Shift this trit into the opposite balance domain.
    fn shift(self) -> Self::Target;
}
