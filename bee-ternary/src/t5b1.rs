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

use crate::{Btrit, RawEncoding, RawEncodingBuf, ShiftTernary, Utrit};
use std::ops::Range;

// Trits Per Byte
const TPB: usize = 5;
// Number required to push a byte between balanced and unbalanced representations
const BAL: i8 = 121;

/// An encoding scheme slice that uses a single byte to represent five trits.
#[repr(transparent)]
pub struct T5B1([()]);

impl T5B1 {
    unsafe fn make(ptr: *const i8, offset: usize, len: usize) -> *const Self {
        let len = (len << 3) | (offset % TPB);
        std::mem::transmute((ptr.add(offset / TPB), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const i8 {
        let byte_offset = (self.len_offset().1 + index) / TPB;
        (self.0.as_ptr() as *const i8).add(byte_offset)
    }

    fn len_offset(&self) -> (usize, usize) {
        (self.0.len() >> 3, self.0.len() & 0b111)
    }
}

fn extract(x: i8, elem: usize) -> Btrit {
    debug_assert!(
        elem < TPB,
        "Attempted to extract invalid element {} from balanced T5B1 trit",
        elem
    );
    Utrit::from_u8((((x as i16 + BAL as i16) / 3i16.pow(elem as u32)) % 3) as u8).shift()
}

fn insert(x: i8, elem: usize, trit: Btrit) -> i8 {
    debug_assert!(
        elem < TPB,
        "Attempted to insert invalid element {} into balanced T5B1 trit",
        elem
    );
    let utrit = trit.shift();
    let ux = x as i16 + BAL as i16;
    let ux = ux + (utrit.into_u8() as i16 - (ux / 3i16.pow(elem as u32)) % 3) * 3i16.pow(elem as u32);
    (ux - BAL as i16) as i8
}

impl RawEncoding for T5B1 {
    type Trit = Btrit;
    type Buf = T5B1Buf;

    fn empty() -> &'static Self {
        unsafe { &*Self::make(&[] as *const _, 0, 0) }
    }

    fn len(&self) -> usize {
        self.len_offset().0
    }

    fn as_i8_slice(&self) -> &[i8] {
        assert!(self.len_offset().1 == 0);
        unsafe { std::slice::from_raw_parts(self as *const _ as *const _, (self.len() + TPB - 1) / TPB) }
    }

    unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        assert!(self.len_offset().1 == 0);
        std::slice::from_raw_parts_mut(self as *mut _ as *mut _, (self.len() + TPB - 1) / TPB)
    }

    unsafe fn get_unchecked(&self, index: usize) -> Self::Trit {
        let b = self.ptr(index).read();
        extract(b, (self.len_offset().1 + index) % TPB)
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Self::Trit) {
        let b = self.ptr(index).read();
        let b = insert(b, (self.len_offset().1 + index) % TPB, trit);
        (self.ptr(index) as *mut i8).write(b);
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(
            self.ptr(range.start),
            (self.len_offset().1 + range.start) % TPB,
            range.end - range.start,
        )
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(
            self.ptr(range.start),
            (self.len_offset().1 + range.start) % TPB,
            range.end - range.start,
        ) as *mut Self)
    }

    fn is_valid(b: i8) -> bool {
        b >= -BAL && b <= BAL
    }

    unsafe fn from_raw_unchecked(b: &[i8], num_trits: usize) -> &Self {
        assert!(num_trits <= b.len() * TPB);
        &*Self::make(b.as_ptr() as *const _, 0, num_trits)
    }

    unsafe fn from_raw_unchecked_mut(b: &mut [i8], num_trits: usize) -> &mut Self {
        assert!(num_trits <= b.len() * TPB);
        &mut *(Self::make(b.as_ptr() as *const _, 0, num_trits) as *mut _)
    }
}

/// An encoding scheme buffer that uses a single byte to represent five trits.
#[derive(Clone)]
pub struct T5B1Buf(Vec<i8>, usize);

impl RawEncodingBuf for T5B1Buf {
    type Slice = T5B1;

    fn new() -> Self {
        Self(Vec::new(), 0)
    }

    fn push(&mut self, trit: <Self::Slice as RawEncoding>::Trit) {
        if self.1 % TPB == 0 {
            self.0.push(insert(0, 0, trit));
        } else {
            let last_index = self.0.len() - 1;
            let b = unsafe { self.0.get_unchecked_mut(last_index) };
            *b = insert(*b, self.1 % TPB, trit);
        }
        self.1 += 1;
    }

    fn pop(&mut self) -> Option<<Self::Slice as RawEncoding>::Trit> {
        let val = if self.1 == 0 {
            return None;
        } else if self.1 % TPB == 1 {
            self.0.pop().map(|b| extract(b, 0))
        } else {
            let last_index = self.0.len() - 1;
            unsafe { Some(extract(*self.0.get_unchecked(last_index), (self.1 + TPB - 1) % TPB)) }
        };
        self.1 -= 1;
        val
    }

    fn as_slice(&self) -> &Self::Slice {
        unsafe { &*Self::Slice::make(self.0.as_ptr() as _, 0, self.1) }
    }

    fn as_slice_mut(&mut self) -> &mut Self::Slice {
        unsafe { &mut *(Self::Slice::make(self.0.as_ptr() as _, 0, self.1) as *mut _) }
    }
}
