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
}

/// A trait implemented by trits that can be shifted between balance domains.
// TODO: Is this a good API?
pub trait ShiftTernary: Sized {
    /// The trit type that results from shifting this trit.
    type Target: ShiftTernary<Target = Self>;

    /// Shift this trit into the opposite balance domain.
    fn shift(self) -> Self::Target;
}
