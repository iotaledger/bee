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

use crate::{Trit, TritBuf};
use std::ops::Range;

pub trait RawEncoding {
    type Trit: Trit;
    type Buf: RawEncodingBuf<Slice = Self>;

    /// Get an empty slice of this encoding
    fn empty() -> &'static Self;

    /// Get the number of trits in this buffer
    fn len(&self) -> usize;

    fn as_i8_slice(&self) -> &[i8];

    unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8];

    /// Get the trit at the given index
    unsafe fn get_unchecked(&self, index: usize) -> Self::Trit;

    /// Set the trit at the given index
    unsafe fn set_unchecked(&mut self, index: usize, trit: Self::Trit);

    /// Get a slice of this slice
    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self;

    /// Get a mutable slice of this slice
    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self;

    /// Decide whether a byte is a valid series of trits in this encoding
    fn is_valid(repr: &i8) -> bool;

    /// Unsafely reinterpret a slice of bytes as trit slice
    unsafe fn from_raw_unchecked(b: &[i8], num_trits: usize) -> &Self;

    /// Unsafely reinterpret a slice of bytes as trit slice
    unsafe fn from_raw_unchecked_mut(b: &mut [i8], num_trits: usize) -> &mut Self;
}

pub trait RawEncodingBuf {
    type Slice: RawEncoding + ?Sized;

    /// Create a new empty buffer
    fn new() -> Self
    where
        Self: Sized;

    /// Create a new buffer containing the given trits
    fn from_trits(trits: &[<Self::Slice as RawEncoding>::Trit]) -> Self
    where
        Self: Sized,
    {
        let mut this = Self::new();
        for trit in trits {
            this.push(*trit);
        }
        this
    }

    /// Push a trit to the back of this buffer
    fn push(&mut self, trit: <Self::Slice as RawEncoding>::Trit);

    /// Pop a trit from the back of this buffer
    fn pop(&mut self) -> Option<<Self::Slice as RawEncoding>::Trit>;

    /// View the trits in this buffer as a slice
    fn as_slice(&self) -> &Self::Slice;

    /// View the trits in this buffer as a mutable slice
    fn as_slice_mut(&mut self) -> &mut Self::Slice;

    /// Convert this encoding into another encoding
    fn into_encoding<T: RawEncodingBuf>(this: TritBuf<Self>) -> TritBuf<T>
    where
        Self: Sized,
        T: RawEncodingBuf,
        T::Slice: RawEncoding<Trit = <Self::Slice as RawEncoding>::Trit>,
    {
        // if TypeId::of::<Self>() == TypeId::of::<T>() {
        //     unsafe { std::mem::transmute(this) }
        // } else {
        this.iter().collect()
        // }
    }
}
