// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Deref, DerefMut, Range};

#[derive(Clone, Copy, Debug)]
pub(crate) struct BcTrit(pub(crate) usize, pub(crate) usize);

impl BcTrit {
    const fn zero() -> Self {
        Self(0, 0)
    }

    pub(crate) fn lo(&self) -> usize {
        self.0
    }

    pub(crate) fn hi(&self) -> usize {
        self.1
    }
}

#[derive(Clone)]
pub(crate) struct BcTritBuf {
    inner: Vec<BcTrit>,
}

impl BcTritBuf {
    pub(crate) fn zeros(len: usize) -> Self {
        Self {
            inner: vec![BcTrit::zero(); len],
        }
    }
}

impl Deref for BcTritBuf {
    type Target = BcTrits;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.inner.deref() as *const [BcTrit] as *const BcTrits) }
    }
}

impl DerefMut for BcTritBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.inner.deref_mut() as *mut [BcTrit] as *mut BcTrits) }
    }
}

#[derive(Clone)]
pub(crate) struct BcTritArr<const N: usize> {
    inner: [BcTrit; N],
}

impl<const N: usize> BcTritArr<N> {
    pub(crate) fn zeros() -> Self {
        Self {
            inner: [BcTrit::zero(); N],
        }
    }

    pub(crate) fn filled(value: usize) -> Self {
        Self {
            inner: [BcTrit(value, value); N],
        }
    }
}

impl<const N: usize> Deref for BcTritArr<N> {
    type Target = BcTrits;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.inner.as_ref() as *const [BcTrit] as *const BcTrits) }
    }
}

impl<const N: usize> DerefMut for BcTritArr<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.inner.as_mut() as *mut [BcTrit] as *mut BcTrits) }
    }
}

#[repr(transparent)]
pub(crate) struct BcTrits {
    inner: [BcTrit],
}

impl BcTrits {
    pub(crate) fn fill(&mut self, value: usize) {
        for BcTrit(hi, lo) in &mut self.inner {
            *lo = value;
            *hi = value;
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }

    pub(crate) fn copy_from_slice(&mut self, slice: &Self) {
        self.inner.copy_from_slice(&slice.inner)
    }

    pub(crate) unsafe fn get_unchecked<I: BcTritsIndex>(&self, index: I) -> &I::Output {
        index.get_unchecked(self)
    }

    pub(crate) unsafe fn get_unchecked_mut<I: BcTritsIndex>(&mut self, index: I) -> &mut I::Output {
        index.get_unchecked_mut(self)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &BcTrit> {
        self.inner.iter()
    }
}

pub(crate) trait BcTritsIndex {
    type Output: ?Sized;

    unsafe fn get_unchecked(self, trits: &BcTrits) -> &Self::Output;
    unsafe fn get_unchecked_mut(self, trits: &mut BcTrits) -> &mut Self::Output;
}

impl BcTritsIndex for usize {
    type Output = BcTrit;

    unsafe fn get_unchecked(self, trits: &BcTrits) -> &Self::Output {
        trits.inner.get_unchecked(self)
    }

    unsafe fn get_unchecked_mut(self, trits: &mut BcTrits) -> &mut Self::Output {
        trits.inner.get_unchecked_mut(self)
    }
}

impl BcTritsIndex for Range<usize> {
    type Output = BcTrits;

    unsafe fn get_unchecked(self, trits: &BcTrits) -> &Self::Output {
        &*(trits.inner.get_unchecked(self) as *const [BcTrit] as *const BcTrits)
    }

    unsafe fn get_unchecked_mut(self, trits: &mut BcTrits) -> &mut Self::Output {
        &mut *(trits.inner.get_unchecked_mut(self) as *mut [BcTrit] as *mut BcTrits)
    }
}
