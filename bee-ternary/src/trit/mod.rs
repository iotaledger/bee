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

pub mod balanced;
pub mod unbalanced;

// Reexports
pub use self::{balanced::Btrit, unbalanced::Utrit};

pub trait Trit:
    Copy + Sized + fmt::Debug + hash::Hash + Into<i8> + Ord + PartialEq + ShiftTernary + TryFrom<i8>
{
    fn checked_increment(self) -> Option<Self>;
    fn zero() -> Self;

    fn as_arbitrary_ref<'a>(&self) -> &'a Self;
}

pub trait ShiftTernary: Sized {
    type Target: ShiftTernary<Target = Self>;

    fn shift(self) -> Self::Target;
}
