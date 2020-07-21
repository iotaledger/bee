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

use crate::ternary::bigint::{binary_representation::U32Repr, endianness::LittleEndian, T243, U384};

use bee_ternary::{Btrit, Utrit};

use lazy_static::lazy_static;

lazy_static! {
    /// T243 balanced trits represented 0.
    pub static ref BTRIT_0: T243<Btrit> = T243::<Btrit>::zero();
    /// T243 balanced trits represented 1.
    pub static ref BTRIT_1: T243<Btrit> = T243::<Btrit>::one();
    /// T243 balanced trits represented -1.
    pub static ref BTRIT_NEG_1: T243<Btrit> = T243::<Btrit>::neg_one();
    /// T243 unbalanced trits represented 0.
    pub static ref UTRIT_0: T243<Utrit> = T243::<Utrit>::zero();
    /// T243 unbalanced trits represented 1.
    pub static ref UTRIT_1: T243<Utrit> = T243::<Utrit>::one();
    /// T243 unbalanced trits represented 2.
    pub static ref UTRIT_2: T243<Utrit> = T243::<Utrit>::two();
    /// T243 unbalanced trits represented U384::max.
    pub static ref UTRIT_U384_MAX: T243<Utrit> = From::from(U384::<LittleEndian, U32Repr>::max());
    /// T243 unbalanced trits represented half of U384::max.
    pub static ref UTRIT_U384_MAX_HALF: T243<Utrit> = {
        let mut u384_max = U384::<LittleEndian, U32Repr>::max();
        u384_max.divide_by_two();
        u384_max.into()
    };
}
