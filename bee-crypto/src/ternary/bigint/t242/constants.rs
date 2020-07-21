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

use crate::ternary::bigint::{binary_representation::U32Repr, endianness::LittleEndian, T242, U384};

use bee_ternary::{Btrit, Utrit};

use lazy_static::lazy_static;

lazy_static! {
    /// T242 balanced trits represented 0.
    pub static ref BTRIT_0: T242<Btrit> = T242::<Btrit>::zero();
    /// T242 balanced trits represented 1.
    pub static ref BTRIT_1: T242<Btrit> = T242::<Btrit>::one();
    /// T242 balanced trits represented -1.
    pub static ref BTRIT_NEG_1: T242<Btrit> = T242::<Btrit>::neg_one();
    /// T242 balanced trits represented maximum value.
    pub static ref BTRIT_MAX: T242<Btrit> = T242::<Btrit>::max();
    /// T242 balanced trits represented minimum value.
    pub static ref BTRIT_MIN: T242<Btrit> = T242::<Btrit>::min();
    /// T242 unbalanced trits represented 0.
    pub static ref UTRIT_0: T242<Utrit> = T242::<Utrit>::zero();
    /// T242 unbalanced trits represented 1.
    pub static ref UTRIT_1: T242<Utrit> = T242::<Utrit>::one();
    /// T242 unbalanced trits represented 2.
    pub static ref UTRIT_2: T242<Utrit> = T242::<Utrit>::two();
    /// T242 unbalanced trits represented U384::max.
    pub static ref UTRIT_U384_MAX: T242<Utrit> = {
        U384::<LittleEndian, U32Repr>::max().into()
    };
    /// T242 unbalanced trits represented half of U384::max.
    pub static ref UTRIT_U384_MAX_HALF: T242<Utrit> = {
        let mut u384_max = U384::<LittleEndian, U32Repr>::max();
        u384_max.divide_by_two();
        u384_max.into()
    };
}
