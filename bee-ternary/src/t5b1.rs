// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Btrit, RawEncoding, RawEncodingBuf, ShiftTernary, Utrit};

use std::ops::Range;

const TRITS_PER_BYTE: usize = 5;
// Number required to push a byte between balanced and unbalanced representations
const BALANCE_DIFF: i8 = 121;

/// An encoding scheme slice that uses a single byte to represent five trits.
#[repr(transparent)]
pub struct T5B1([()]);

impl T5B1 {
    unsafe fn make(ptr: *const i8, offset: usize, len: usize) -> *const Self {
        let len = (len << 3) | (offset % TRITS_PER_BYTE);
        std::mem::transmute((ptr.add(offset / TRITS_PER_BYTE), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const i8 {
        let byte_offset = (self.len_offset().1 + index) / TRITS_PER_BYTE;
        (self.0.as_ptr() as *const i8).add(byte_offset)
    }

    fn len_offset(&self) -> (usize, usize) {
        (self.0.len() >> 3, self.0.len() & 0b111)
    }
}

fn extract(x: i8, elem: usize) -> Btrit {
    debug_assert!(
        elem < TRITS_PER_BYTE,
        "Attempted to extract invalid element {} from balanced T5B1 trit",
        elem
    );
    Utrit::from_u8((((x as i16 + BALANCE_DIFF as i16) / 3i16.pow(elem as u32)) % 3) as u8).shift()
}

fn insert(x: i8, elem: usize, trit: Btrit) -> i8 {
    debug_assert!(
        elem < TRITS_PER_BYTE,
        "Attempted to insert invalid element {} into balanced T5B1 trit",
        elem
    );
    let utrit = trit.shift();
    let ux = x as i16 + BALANCE_DIFF as i16;
    let ux = ux + (utrit.into_u8() as i16 - (ux / 3i16.pow(elem as u32)) % 3) * 3i16.pow(elem as u32);
    (ux - BALANCE_DIFF as i16) as i8
}

impl RawEncoding for T5B1 {
    type Trit = Btrit;
    type Buf = T5B1Buf;

    const TRITS_PER_BYTE: usize = TRITS_PER_BYTE;

    fn empty() -> &'static Self {
        unsafe { &*Self::make(&[] as *const _, 0, 0) }
    }

    fn len(&self) -> usize {
        self.len_offset().0
    }

    fn as_i8_slice(&self) -> &[i8] {
        assert!(self.len_offset().1 == 0);
        unsafe {
            std::slice::from_raw_parts(
                self as *const _ as *const _,
                (self.len() + TRITS_PER_BYTE - 1) / TRITS_PER_BYTE,
            )
        }
    }

    unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        assert!(self.len_offset().1 == 0);
        std::slice::from_raw_parts_mut(
            self as *mut _ as *mut _,
            (self.len() + TRITS_PER_BYTE - 1) / TRITS_PER_BYTE,
        )
    }

    unsafe fn get_unchecked(&self, index: usize) -> Self::Trit {
        let b = self.ptr(index).read();
        extract(b, (self.len_offset().1 + index) % TRITS_PER_BYTE)
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Self::Trit) {
        let b = self.ptr(index).read();
        let b = insert(b, (self.len_offset().1 + index) % TRITS_PER_BYTE, trit);
        (self.ptr(index) as *mut i8).write(b);
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(
            self.ptr(range.start),
            (self.len_offset().1 + range.start) % TRITS_PER_BYTE,
            range.end - range.start,
        )
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(
            self.ptr(range.start),
            (self.len_offset().1 + range.start) % TRITS_PER_BYTE,
            range.end - range.start,
        ) as *mut Self)
    }

    fn is_valid(b: i8) -> bool {
        b >= -BALANCE_DIFF && b <= BALANCE_DIFF
    }

    unsafe fn from_raw_unchecked(b: &[i8], num_trits: usize) -> &Self {
        assert!(num_trits <= b.len() * TRITS_PER_BYTE);
        &*Self::make(b.as_ptr() as *const _, 0, num_trits)
    }

    unsafe fn from_raw_unchecked_mut(b: &mut [i8], num_trits: usize) -> &mut Self {
        assert!(num_trits <= b.len() * TRITS_PER_BYTE);
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

    fn with_capacity(cap: usize) -> Self {
        let cap = (cap / TRITS_PER_BYTE) + (cap % TRITS_PER_BYTE != 0) as usize;
        Self(Vec::with_capacity(cap), 0)
    }

    fn clear(&mut self) {
        self.0.clear();
        self.1 = 0;
    }

    fn push(&mut self, trit: <Self::Slice as RawEncoding>::Trit) {
        if self.1 % TRITS_PER_BYTE == 0 {
            self.0.push(insert(0, 0, trit));
        } else {
            let last_index = self.0.len() - 1;
            let b = unsafe { self.0.get_unchecked_mut(last_index) };
            *b = insert(*b, self.1 % TRITS_PER_BYTE, trit);
        }
        self.1 += 1;
    }

    fn pop(&mut self) -> Option<<Self::Slice as RawEncoding>::Trit> {
        let val = if self.1 == 0 {
            return None;
        } else if self.1 % TRITS_PER_BYTE == 1 {
            self.0.pop().map(|b| extract(b, 0))
        } else {
            let last_index = self.0.len() - 1;
            unsafe {
                Some(extract(
                    *self.0.get_unchecked(last_index),
                    (self.1 + TRITS_PER_BYTE - 1) % TRITS_PER_BYTE,
                ))
            }
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

    fn capacity(&self) -> usize {
        self.0.capacity() * TRITS_PER_BYTE
    }
}
