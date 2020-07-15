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

//! Split integers to access the high and low parts of an integer.

/// Represents a split integer into high and low parts.
pub(crate) trait SplitInteger: Copy {
    /// The type of the high part of the integer.
    type High;
    /// The type of the low part of the integer.
    type Low;

    /// Returns the high part of the integer.
    fn hi(self) -> Self::High;
    /// Returns the low part of the integer.
    fn lo(self) -> Self::Low;
}

impl SplitInteger for i64 {
    type High = i32;
    type Low = u32;

    fn hi(self) -> Self::High {
        (self >> 32) as Self::High
    }

    fn lo(self) -> Self::Low {
        self as Self::Low
    }
}

impl SplitInteger for u64 {
    type High = u32;
    type Low = u32;

    fn hi(self) -> Self::High {
        (self >> 32) as Self::High
    }

    fn lo(self) -> Self::Low {
        self as Self::Low
    }
}

#[cfg(test)]
mod tests {
    use super::SplitInteger;

    macro_rules! test_split_integers {
        ( $( [$fname:ident, $src:expr, $dst:expr] ),+ $(,)? ) => {
            $(
                #[test]
                fn $fname() {
                    assert_eq!($src, $dst);
                }
            )+
        }
    }

    test_split_integers!(
        // i64
        [split_i64_hi_one_is_zero, 1i64.hi(), 0i32],
        [split_i64_lo_one_is_one, 1i64.lo(), 1u32],
        [split_i64_hi_max_is_max, i64::max_value().hi(), i32::max_value()],
        [split_i64_lo_max_is_max, i64::max_value().lo(), u32::max_value()],
        [split_i64_hi_min_is_min, i64::min_value().hi(), i32::min_value()],
        [split_i64_lo_min_is_zero, i64::min_value().lo(), 0u32],
        [split_i64_hi_neg_one_is_neg_one, (-1i64).hi(), -1i32],
        [split_i64_lo_neg_one_is_max, (-1i64).lo(), u32::max_value()],
        // u64
        [split_u64_hi_one_is_zero, 1u64.hi(), 0u32],
        [split_u64_lo_one_is_one, 1u64.lo(), 1u32],
        [split_u64_hi_max_is_max, u64::max_value().hi(), u32::max_value()],
        [split_u64_lo_max_is_max, u64::max_value().lo(), u32::max_value()],
        [split_u64_hi_min_is_min, u64::min_value().hi(), 0u32],
        [split_u64_lo_min_is_zero, u64::min_value().lo(), 0u32],
    );
}
