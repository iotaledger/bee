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

use std::slice::SliceIndex;

#[derive(Clone)]
pub struct BCTritBuf {
    lo: Vec<usize>,
    hi: Vec<usize>,
}

impl BCTritBuf {
    pub fn as_slice(&self) -> BCTritRef<'_, [usize]> {
        BCTritRef {
            lo: &self.lo,
            hi: &self.hi,
        }
    }

    pub fn as_slice_mut(&mut self) -> BCTritMut<'_, [usize]> {
        BCTritMut {
            lo: &mut self.lo,
            hi: &mut self.hi,
        }
    }

    pub fn fill(&mut self, value: usize) {
        for (lo, hi) in self.lo.iter_mut().zip(self.hi.iter_mut()) {
            *lo = value;
            *hi = value;
        }
    }

    pub fn filled(value: usize, len: usize) -> Self {
        BCTritBuf {
            lo: vec![value; len],
            hi: vec![value; len],
        }
    }

    pub fn zeros(len: usize) -> Self {
        Self::filled(0, len)
    }

    pub fn len(&self) -> usize {
        self.lo.len()
    }

    pub fn lo(&self) -> &[usize] {
        &self.lo
    }

    pub fn hi(&self) -> &[usize] {
        &self.hi
    }

    pub fn lo_mut(&mut self) -> &mut [usize] {
        &mut self.lo
    }

    pub fn hi_mut(&mut self) -> &mut [usize] {
        &mut self.hi
    }

    pub unsafe fn get_unchecked<I: SliceIndex<[usize]> + Clone>(&self, index: I) -> BCTritRef<I::Output> {
        BCTritRef {
            lo: self.lo.get_unchecked(index.clone()),
            hi: self.hi.get_unchecked(index),
        }
    }

    pub unsafe fn get_unchecked_mut<I: SliceIndex<[usize]> + Clone>(&mut self, index: I) -> BCTritMut<I::Output> {
        BCTritMut {
            lo: self.lo.get_unchecked_mut(index.clone()),
            hi: self.hi.get_unchecked_mut(index),
        }
    }
}

pub struct BCTritRef<'a, T: ?Sized> {
    pub lo: &'a T,
    pub hi: &'a T,
}

pub struct BCTritMut<'a, T: ?Sized> {
    pub lo: &'a mut T,
    pub hi: &'a mut T,
}

impl<'a> BCTritMut<'a, [usize]> {
    pub fn copy_from_slice(&mut self, slice: BCTritRef<'_, [usize]>) {
        self.lo.copy_from_slice(slice.lo);
        self.hi.copy_from_slice(slice.hi);
    }
}
