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

use std::ops::{Deref, DerefMut, Range};

#[derive(Clone, Copy, Debug)]
pub struct BCTrit(pub usize, pub usize);

impl BCTrit {
    const fn zero() -> Self {
        Self(0, 0)
    }

    pub fn lo(&self) -> usize {
        self.0
    }

    pub fn hi(&self) -> usize {
        self.1
    }
}

#[derive(Clone)]
pub struct BCTritBuf {
    inner: Vec<BCTrit>,
}

impl BCTritBuf {
    pub fn zeros(len: usize) -> Self {
        Self {
            inner: vec![BCTrit::zero(); len],
        }
    }

    pub fn filled(value: usize, len: usize) -> Self {
        Self {
            inner: vec![BCTrit(value, value); len],
        }
    }
}

impl Deref for BCTritBuf {
    type Target = BCTrits;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.inner.deref() as *const [BCTrit] as *const BCTrits) }
    }
}

impl DerefMut for BCTritBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.inner.deref_mut() as *mut [BCTrit] as *mut BCTrits) }
    }
}

#[repr(transparent)]
pub struct BCTrits {
    inner: [BCTrit],
}

impl BCTrits {
    pub fn fill(&mut self, value: usize) {
        for BCTrit(hi, lo) in &mut self.inner {
            *lo = value;
            *hi = value;
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn copy_from_slice(&mut self, slice: &Self) {
        self.inner.copy_from_slice(&slice.inner)
    }

    pub unsafe fn get_unchecked<I: BCTritsIndex>(&self, index: I) -> &I::Output {
        index.get_unchecked(self)
    }

    pub unsafe fn get_unchecked_mut<I: BCTritsIndex>(&mut self, index: I) -> &mut I::Output {
        index.get_unchecked_mut(self)
    }

    pub fn iter(&self) -> impl Iterator<Item = &BCTrit> {
        self.inner.iter()
    }
}

pub trait BCTritsIndex {
    type Output: ?Sized;

    unsafe fn get_unchecked(self, trits: &BCTrits) -> &Self::Output;
    unsafe fn get_unchecked_mut(self, trits: &mut BCTrits) -> &mut Self::Output;
}

impl BCTritsIndex for usize {
    type Output = BCTrit;

    unsafe fn get_unchecked(self, trits: &BCTrits) -> &Self::Output {
        trits.inner.get_unchecked(self)
    }

    unsafe fn get_unchecked_mut(self, trits: &mut BCTrits) -> &mut Self::Output {
        trits.inner.get_unchecked_mut(self)
    }
}

impl BCTritsIndex for Range<usize> {
    type Output = BCTrits;

    unsafe fn get_unchecked(self, trits: &BCTrits) -> &Self::Output {
        &*(trits.inner.get_unchecked(self) as *const [BCTrit] as *const BCTrits)
    }

    unsafe fn get_unchecked_mut(self, trits: &mut BCTrits) -> &mut Self::Output {
        &mut *(trits.inner.get_unchecked_mut(self) as *mut [BCTrit] as *mut BCTrits)
    }
}
