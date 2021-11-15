// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{trit::ShiftTernary, Btrit, RawEncoding, RawEncodingBuf, Trit};

use std::{hash, marker::PhantomData, ops::Range};

/// An encoding scheme slice that uses a single byte to represent one trit.
#[repr(transparent)]
pub struct T1B1<T: Trit = Btrit> {
    _phantom: PhantomData<T>,
    inner: [()],
}

impl<T: Trit> T1B1<T> {
    unsafe fn make(ptr: *const T, offset: usize, len: usize) -> *const Self {
        std::mem::transmute((ptr.add(offset), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const T {
        (self.inner.as_ptr() as *const T).add(index)
    }

    pub(crate) fn as_raw_slice(&self) -> &[T] {
        unsafe { &*(Self::make(self.ptr(0), 0, self.len()) as *const _) }
    }

    pub(crate) fn as_raw_slice_mut(&mut self) -> &mut [T] {
        unsafe { &mut *(Self::make(self.ptr(0), 0, self.len()) as *mut _) }
    }
}

impl<T: Trit> hash::Hash for T1B1<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_raw_slice().hash(state);
    }
}

impl<T> RawEncoding for T1B1<T>
where
    T: Trit,
{
    type Trit = T;
    type Buf = T1B1Buf<T>;

    fn empty() -> &'static Self {
        unsafe { &*Self::make(&[] as *const _, 0, 0) }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn as_i8_slice(&self) -> &[i8] {
        unsafe { &*(Self::make(self.ptr(0), 0, self.len()) as *const _) }
    }

    unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        &mut *(Self::make(self.ptr(0), 0, self.len()) as *mut _)
    }

    unsafe fn get_unchecked(&self, index: usize) -> Self::Trit {
        self.ptr(index).read()
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Self::Trit) {
        (self.ptr(index) as *mut T).write(trit);
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(self.ptr(0), range.start, range.end - range.start)
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(self.ptr(0), range.start, range.end - range.start) as *mut _)
    }

    fn is_valid(b: i8) -> bool {
        TryInto::<T>::try_into(b).is_ok()
    }

    unsafe fn from_raw_unchecked(b: &[i8], num_trits: usize) -> &Self {
        assert!(num_trits <= b.len());
        &*Self::make(b.as_ptr() as *const _, 0, num_trits)
    }

    unsafe fn from_raw_unchecked_mut(b: &mut [i8], num_trits: usize) -> &mut Self {
        assert!(num_trits <= b.len());
        &mut *(Self::make(b.as_ptr() as *const _, 0, num_trits) as *mut _)
    }
}

/// An encoding scheme buffer that uses a single byte to represent one trit.
#[derive(Clone)]
pub struct T1B1Buf<T: Trit = Btrit>(Vec<T>);

impl<T> T1B1Buf<T>
where
    T: Trit,
    <T as ShiftTernary>::Target: Trit,
{
    pub(crate) fn into_shifted(self) -> T1B1Buf<T::Target> {
        // Shift each trit, cast it to i8, and update the inner buffer.
        // This puts the inner buffer into an incorrect state!
        let mut trit_buf = self;
        unsafe {
            trit_buf.as_slice_mut().as_i8_slice_mut().iter_mut().for_each(|t| {
                // Unwrapping is safe because the bytes are coming from
                // within the trit buffer.
                let trit: T = (*t)
                    .try_into()
                    .unwrap_or_else(|_| unreachable!("Unreachable because input bytes are guaranteed to be correct"));
                let shifted_trit = trit.shift();
                *t = shifted_trit.into();
            });
        }

        // Take ownership of the inner vector and cast it to a `Vec<T::Target>`
        let raw_trits = std::mem::ManuallyDrop::new(trit_buf.0);

        let p = raw_trits.as_ptr();
        let len = raw_trits.len();
        let cap = raw_trits.capacity();

        let raw_shifted_trits = unsafe { Vec::from_raw_parts(p as *const i8 as *mut _, len, cap) };

        T1B1Buf(raw_shifted_trits)
    }
}

impl<T> RawEncodingBuf for T1B1Buf<T>
where
    T: Trit,
{
    type Slice = T1B1<T>;

    fn new() -> Self {
        Self(Vec::new())
    }

    fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn push(&mut self, trit: <Self::Slice as RawEncoding>::Trit) {
        self.0.push(trit);
    }

    fn pop(&mut self) -> Option<<Self::Slice as RawEncoding>::Trit> {
        self.0.pop()
    }

    fn as_slice(&self) -> &Self::Slice {
        unsafe { &*Self::Slice::make(self.0.as_ptr() as _, 0, self.0.len()) }
    }

    fn as_slice_mut(&mut self) -> &mut Self::Slice {
        unsafe { &mut *(Self::Slice::make(self.0.as_ptr() as _, 0, self.0.len()) as *mut _) }
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }
}
