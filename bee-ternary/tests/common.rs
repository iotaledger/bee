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

use bee_ternary::*;
use rand::prelude::*;
use std::{convert::TryFrom, ops::Range};

pub fn gen_trit() -> i8 {
    (thread_rng().gen::<u8>() % 3) as i8 - 1
}

pub fn gen_buf<T: raw::RawEncodingBuf>(len: Range<usize>) -> (TritBuf<T>, Vec<i8>) {
    let len = thread_rng().gen_range(len.start, len.end);
    let trits = (0..len).map(|_| gen_trit()).collect::<Vec<_>>();
    (
        trits
            .iter()
            .map(|t| <T::Slice as raw::RawEncoding>::Trit::try_from(*t).ok().unwrap())
            .collect(),
        trits,
    )
}

pub fn gen_buf_unbalanced<T: raw::RawEncodingBuf>(len: Range<usize>) -> (TritBuf<T>, Vec<i8>) {
    let len = thread_rng().gen_range(len.start, len.end);
    let trits = (0..len).map(|_| gen_trit() + 1).collect::<Vec<_>>();
    (
        trits
            .iter()
            .map(|t| <T::Slice as raw::RawEncoding>::Trit::try_from(*t).ok().unwrap())
            .collect(),
        trits,
    )
}

// Not exactly fuzzing, just doing something a lot
pub fn fuzz(n: usize, mut f: impl FnMut()) {
    (0..n).for_each(|_| f());
}
