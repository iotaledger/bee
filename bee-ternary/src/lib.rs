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

//! A general-purpose ternary manipulation, translation and encoding crate.
//!
//! # Features
//!
//! - Creation of trit and tryte buffers with multiple encodings
//! - Safe encoding API that allows the efficient manipulation and sharing of trit and tryte buffers and slices
//! - Mutation of trit buffers and slices
//! - Ternary BigInt implementation
//! - Balanced and unbalanced ternary
//! - `serde` support
//!
//! # Encodings
//!
//! This crate includes support for many different trit encodings. Encodings allow the trading off
//! of different features against each other.
//!
//! [`T1B1`] is the canonical default encoding and represents every trit with a single byte of
//! memory. It is the fastest encoding to manipulate since no bitwise operations are necessary to
//! pack and unpack it from memory during manipulation. As a result of this, it also permits
//! certain extra features like mutable chunking and accessing its contents through ordinary
//! slices.
//!
//! [`T3B1`] is also commonly used. It provides good compression and has the advantage that it has
//! an identical bit representation as a [`Tryte`] slice. For this reason, it is the only encoding
//! that can be converted to a tryte slice with no overhead.
//!
//! [`T5B1`] is the most compressed encoding. It provides very high storage densities (almost
//! optimal, in fact) and is the densest encoding supported by this crate.
//!
//! It is likely that one of the 3 encodings above will suit your requirements. In addition, this
//! crate also supports [`T2B1`] and [`T4B1`] for completeness.
//!
//! # Byte Alignment
//!
//! This crate supports creating sub-slices of trit slices. To enable this, it stores extra
//! metadata along with a trit slice in order to correct identify the index of a buffer it starts
//! on. With compressed encodings, such as [`T3B1`], that starting index (and, indeed, the end
//! index) may not fall exactly on a byte boundary.
//!
//! This crate makes a best attempt at avoiding the negative ramifications of this fact, but sadly
//! some still leak through into the API. For example, some methods may panic if a slice does not
//! have a byte-aligned starting index or otherwise does not fulfil certain invariants. However,
//! all panicking behaviours are documented on each method such that you can easily avoid
//! circumstances like this.
//!
//! When the documentation refers to 'byte alignment', it is referring specifically to whether the
//! starting index is a multiple of the compression factor. For example a byte-aligned [`T3B1`]
//! buffer will always start on an index of the *original* buffer that is a multiple of 3.

#![deny(missing_docs)]

use std::slice;

/// Conversions between to and from standard types.
pub mod convert;
/// Types and traits that allow the implementation of new encoding formats.
pub mod raw;
/// The [`T1B1`] and [`T1B1Buf`] encodings.
pub mod t1b1;
/// The [`T2B1`] and [`T2B1Buf`] encodings.
pub mod t2b1;
/// The [`T3B1`] and [`T3B1Buf`] encodings.
pub mod t3b1;
/// The [`T4B1`] and [`T4B1Buf`] encodings.
pub mod t4b1;
/// The [`T5B1`] and [`T5B1Buf`] encodings.
pub mod t5b1;
/// Types and traits used to represent trits, both balanced and unbalanced.
pub mod trit;
/// Types and traits used to represent trytes and buffers of trytes.
pub mod tryte;

#[cfg(feature = "serde1")]
mod serde;

use crate::raw::{RawEncoding, RawEncodingBuf};
use std::{
    any,
    borrow::{Borrow, BorrowMut},
    cmp::{self, Ordering},
    convert::TryFrom,
    error, fmt, hash,
    iter::FromIterator,
    ops::{
        Deref, DerefMut, Index, IndexMut, Neg, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
    },
};

// Reexports
pub use crate::{
    t1b1::{T1B1Buf, T1B1},
    t2b1::{T2B1Buf, T2B1},
    t3b1::{T3B1Buf, T3B1},
    t4b1::{T4B1Buf, T4B1},
    t5b1::{T5B1Buf, T5B1},
    trit::{Btrit, ShiftTernary, Trit, Utrit},
    tryte::{Tryte, TryteBuf},
};

/// An error that may be produced as a result of fallible conversions.
#[derive(Debug)]
pub enum Error {
    /// A value that does not represent a valid ternary representation was encountered.
    InvalidRepr,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidRepr => write!(f, "invalid representation"),
        }
    }
}

impl error::Error for Error {}

/// A type that represents a buffer of trits of unknown length.
///
/// This type is roughly analogous to `[T]` or [`str`]. It is an unsized type and hence is rarely
/// used directly. Instead, it's more common to see it used from behind a reference (in a similar
/// manner to `&[T]` and `&str`.
#[derive(Hash)]
#[repr(transparent)]
pub struct Trits<T: RawEncoding + ?Sized = T1B1<Btrit>>(T);

impl<T> Trits<T>
where
    T: RawEncoding + ?Sized,
{
    /// Create an empty trit slice.
    pub fn empty() -> &'static Self {
        unsafe { &*(T::empty() as *const _ as *const Self) }
    }

    /// Interpret an (`std::i8`) slice as a trit slice with the given encoding without first
    /// checking that the slice is valid in the given encoding. The `num_trits` parameter is used
    /// to specify the exact length, in trits, that the slice should be taken to have. Providing a
    /// slice that is not valid for this encoding is undefined behaviour.
    ///
    /// # Panics
    ///
    /// This function will panic if `num_trits` is more than can be represented with the slice in
    /// the given encoding.
    ///
    /// # Safety
    ///
    /// This function must only be called with an [`i8`] slice that is valid for this trit encoding
    /// given the specified `num_trits` length. Right now, this validity is not well-defined and so
    /// it is suggested that only [`i8`] slices created from existing trit slices or trit buffers
    /// be used. Calling this function with an invalid [`i8`] slice is undefined behaviour.
    pub unsafe fn from_raw_unchecked(raw: &[i8], num_trits: usize) -> &Self {
        debug_assert!(
            raw.iter().copied().all(T::is_valid),
            "Invalid i8 slice used to create trit slice"
        );
        &*(T::from_raw_unchecked(raw, num_trits) as *const _ as *const _)
    }

    /// Interpret a mutable (`std::i8`) slice as a mutable trit slice with the given encoding
    /// without first checking that the slice is valid in the given encoding. The `num_trits`
    /// parameter is used to specify the exact length, in trits, that the slice should be taken to
    /// have. Providing a slice that is not valid for this encoding is undefined behaviour.
    ///
    /// # Panics
    ///
    /// This function will panic if `num_trits` is more than can be represented with the slice in
    /// the given encoding.
    ///
    /// # Safety
    ///
    /// This function must only be called with an [`i8`] slice that is valid for this trit encoding
    /// given the specified `num_trits` length. Right now, this validity is not well-defined and so
    /// it is suggested that only [`i8`] slices created from existing trit slices or trit buffers
    /// be used. Calling this function with an invalid [`i8`] slice is undefined behaviour.
    pub unsafe fn from_raw_unchecked_mut(raw: &mut [i8], num_trits: usize) -> &mut Self {
        debug_assert!(
            raw.iter().copied().all(T::is_valid),
            "Invalid i8 slice used to create trit slice"
        );
        &mut *(T::from_raw_unchecked_mut(raw, num_trits) as *mut _ as *mut _)
    }

    /// Interpret an (`std::i8`) slice as a trit slice with the given encoding, checking to ensure
    /// that the slice is valid in the given encoding. The `num_trits` parameter is used to specify
    /// the exact length, in trits, that the slice should be taken to have.
    ///
    /// # Panics
    ///
    /// This function will panic if `num_trits` is more than can be represented with the slice in
    /// the given encoding.
    pub fn try_from_raw(raw: &[i8], num_trits: usize) -> Result<&Self, Error> {
        if raw.iter().copied().all(T::is_valid) {
            Ok(unsafe { Self::from_raw_unchecked(raw, num_trits) })
        } else {
            Err(Error::InvalidRepr)
        }
    }

    /// Interpret a mutable (`std::i8`) slice as a mutable trit slice with the given encoding,
    /// checking to ensure that the slice is valid in the given encoding. The `num_trits` parameter
    /// is used to specify the exact length, in trits, that the slice should be taken to have.
    ///
    /// # Panics
    ///
    /// This function will panic if `num_trits` is more than can be represented with the slice in
    /// the given encoding.
    pub fn try_from_raw_mut(raw: &mut [i8], num_trits: usize) -> Result<&mut Self, Error> {
        if raw.iter().copied().all(T::is_valid) {
            Ok(unsafe { Self::from_raw_unchecked_mut(raw, num_trits) })
        } else {
            Err(Error::InvalidRepr)
        }
    }

    /// Returns `true` if the trit slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of trits in this trit slice.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Interpret this slice as an (`std::i8`) slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is not byte-aligned
    pub fn as_i8_slice(&self) -> &[i8] {
        self.0.as_i8_slice()
    }

    /// Interpret this slice as a mutable (`std::i8`) slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is not byte-aligned
    ///
    /// # Safety
    ///
    /// This function is marked `unsafe` because modification of the trit slice in a manner that is
    /// not valid for this encoding is undefined behaviour.
    pub unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        self.0.as_i8_slice_mut()
    }

    /// Fetch the trit at the given index of this trit slice without first checking whether the
    /// index is in bounds. Providing an index that is not less than the length of this slice is
    /// undefined behaviour.
    ///
    /// This is perhaps the 'least bad' `unsafe` function in this crate: not because any form of
    /// undefined behaviour is better or worse than another (after all, the point of undefined
    /// behaviour is that it is undefined) but because it's the easiest to use correctly.
    ///
    /// # Safety
    ///
    /// An index with a value less then the result of [`Trits::len`] must be used. Any other value
    /// is undefined behaviour.
    pub unsafe fn get_unchecked(&self, index: usize) -> T::Trit {
        debug_assert!(
            index < self.0.len(),
            "Attempt to get trit at index {}, but length of slice is {}",
            index,
            self.len(),
        );
        self.0.get_unchecked(index)
    }

    /// Set the trit at the given index of this trit slice without first checking whether the
    /// index is in bounds. Providing an index that is not less than the length of this slice is
    /// undefined behaviour.
    ///
    /// This is perhaps the 'least bad' `unsafe` function in this crate: not because any form of
    /// undefined behaviour is better or worse than another (after all, the point of undefined
    /// behaviour is that it is undefined) but because it's the easiest to use correctly.
    ///
    /// # Safety
    ///
    /// An index with a value less then the result of [`Trits::len`] must be used. Any other value
    /// is undefined behaviour.
    pub unsafe fn set_unchecked(&mut self, index: usize, trit: T::Trit) {
        debug_assert!(
            index < self.0.len(),
            "Attempt to set trit at index {}, but length of slice is {}",
            index,
            self.len(),
        );
        self.0.set_unchecked(index, trit);
    }

    /// Fetch the trit at the given index of this trit slice, if the index is valid.
    pub fn get(&self, index: usize) -> Option<T::Trit> {
        if index < self.0.len() {
            unsafe { Some(self.get_unchecked(index)) }
        } else {
            None
        }
    }

    /// Set the trit at the given index of this mutable trit slice, if the index is valid.
    ///
    /// # Panics
    ///
    /// This function will panic if the index is not less than the length of this slice.
    // TODO: Should we return `Option<()>` instead?
    pub fn set(&mut self, index: usize, trit: T::Trit) {
        assert!(
            index < self.0.len(),
            "Attempt to set trit at index {}, but length of slice is {}",
            index,
            self.len(),
        );
        unsafe { self.set_unchecked(index, trit) };
    }

    /// Returns an iterator over the trits in this slice.
    ///
    /// Using this function is significantly faster than calling [`Trits::get`] in a loop and
    /// should be used where possible.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = T::Trit> + ExactSizeIterator<Item = T::Trit> + '_ {
        (0..self.0.len()).map(move |idx| unsafe { self.0.get_unchecked(idx) })
    }

    /// Returns a subslice of this slice with the given range of trits.
    ///
    /// # Panics
    ///
    /// This function will panic if called with a range that contains indices outside this slice,
    /// or the start of the range is greater than its end.
    pub fn subslice(&self, range: Range<usize>) -> &Self {
        assert!(
            range.end >= range.start && range.end <= self.len(),
            "Sub-slice range must be within the bounds of the source trit slice",
        );
        unsafe { &*(self.0.slice_unchecked(range) as *const _ as *const Self) }
    }

    /// Returns a mutable subslice of this mutable slice with the given range of trits.
    ///
    /// # Panics
    ///
    /// This function will panic if called with a range that contains indices outside this slice,
    /// or the start of the range is greater than its end.
    pub fn subslice_mut(&mut self, range: Range<usize>) -> &mut Self {
        assert!(
            range.end >= range.start && range.end <= self.len(),
            "Sub-slice range must be within the bounds of the source trit slice",
        );
        unsafe { &mut *(self.0.slice_unchecked_mut(range) as *mut _ as *mut Self) }
    }

    /// Copy the trits from a trit slice into this mutable trit slice (the encoding need not be
    /// equivalent).
    ///
    /// # Panics
    ///
    /// This function will panic if the length of the slices are different
    pub fn copy_from<U: RawEncoding<Trit = T::Trit> + ?Sized>(&mut self, trits: &Trits<U>) {
        assert!(
            self.len() == trits.len(),
            "Source trit slice must be the same length as target"
        );
        for (i, trit) in trits.iter().enumerate() {
            unsafe {
                self.set_unchecked(i, trit);
            }
        }
    }

    /// Fill this mutable trit slice with copied of the given trit.
    pub fn fill(&mut self, trit: T::Trit) {
        for i in 0..self.len() {
            unsafe {
                self.set_unchecked(i, trit);
            }
        }
    }

    /// Copy the contents of this trit slice into a new [`TritBuf`] with the same encoding. This
    /// function is analogous to `to_vec` method implemented on ordinary slices.
    pub fn to_buf<U: RawEncodingBuf<Slice = T>>(&self) -> TritBuf<U> {
        // TODO: A faster impl than this!
        self.iter().collect()
    }

    /// Return an iterator over distinct, non-overlapping subslices of this trit slice, each with
    /// the given chunk length. If the length of the trit slice is not a multiple of the given
    /// chunk length, the last slice provided by the iterator will be smaller to compensate.
    ///
    /// # Panics
    ///
    /// This function will panic if the given chunk length is `0`.
    pub fn chunks(
        &self,
        chunk_len: usize,
    ) -> impl DoubleEndedIterator<Item = &Self> + ExactSizeIterator<Item = &Self> + '_ {
        assert!(chunk_len > 0, "Chunk length must be non-zero");
        (0..self.len())
            .step_by(chunk_len)
            .map(move |i| &self[i..(i + chunk_len).min(self.len())])
    }

    /// Encode the contents of this trit slice into a `TritBuf` with a different encoding.
    pub fn encode<U>(&self) -> TritBuf<U>
    where
        U: RawEncodingBuf,
        U::Slice: RawEncoding<Trit = T::Trit>,
    {
        self.iter().collect()
    }
}

impl<T> Trits<T>
where
    T: RawEncoding<Trit = Btrit> + ?Sized,
{
    /// Returns an iterator over the trytes represented within this slice.
    ///
    /// For encodings that are representation-compatible with trytes, such as [`T3B1`], use
    /// [`Trits::as_trytes`] instead since it is faster and more capable.
    pub fn iter_trytes(&self) -> impl DoubleEndedIterator<Item = Tryte> + ExactSizeIterator<Item = Tryte> + '_ {
        assert!(self.len() % 3 == 0, "Trit slice length must be a multiple of 3");
        self.chunks(3)
            .map(|trits| Tryte::from_trits([trits.get(0).unwrap(), trits.get(1).unwrap(), trits.get(2).unwrap()]))
    }

    /// Negate each trit in this buffer.
    ///
    /// This has the effect of making the trit buffer negative when expressed in numeric form.
    pub fn negate(&mut self) {
        for i in 0..self.len() {
            unsafe {
                let t = self.get_unchecked(i);
                self.set_unchecked(i, -t);
            }
        }
    }
}

/// These functions are only implemented for trit slices with the [`T1B1`] encoding because other
/// encodings are compressed and do not support handing out references to their internal trits.
/// [`T1B1`] is an exception because its trits are strictly byte-aligned.
///
/// This fact also implies that [`T1B1`] is the fastest encoding for general-purpose manipulation
/// of trits.
impl<T: Trit> Trits<T1B1<T>> {
    /// View this trit slice as an ordinary slice of trits.
    pub fn as_raw_slice(&self) -> &[T] {
        self.0.as_raw_slice()
    }

    /// View this mutable trit slice as an ordinary slice of mutable trits.
    pub fn as_raw_slice_mut(&mut self) -> &mut [T] {
        self.0.as_raw_slice_mut()
    }

    /// Return an iterator over distinct, non-overlapping mutable subslices of this mutable trit
    /// slice, each with the given chunk length. If the length of the trit slice is not a multiple
    /// of the given chunk length, the last slice provided by the iterator will be smaller to compensate.
    ///
    /// # Panics
    ///
    /// This function will panic if the given chunk length is `0`.
    // Q: Why isn't this method on Trits<T>?
    // A: Because overlapping slice lifetimes make this unsound on squashed encodings
    pub fn chunks_mut(&mut self, chunk_len: usize) -> impl Iterator<Item = &mut Self> + '_ {
        assert!(chunk_len > 0, "Chunk length must be non-zero");
        (0..self.len()).step_by(chunk_len).scan(self, move |this, _| {
            let idx = chunk_len.min(this.len());
            let (a, b) = Trits::split_at_mut(this, idx);
            *this = b;
            Some(a)
        })
    }

    /// Divides this mutable slice into two mutually exclusive mutable slices at the given index.
    ///
    /// The first slice will contain the indices within the range `0..mid` and the second `mid..len`.
    fn split_at_mut<'a>(this: &mut &'a mut Self, mid: usize) -> (&'a mut Self, &'a mut Self) {
        assert!(
            mid <= this.len(),
            "Cannot split at an index outside the trit slice bounds"
        );
        (
            unsafe { &mut *(this.0.slice_unchecked_mut(0..mid) as *mut _ as *mut Self) },
            unsafe { &mut *(this.0.slice_unchecked_mut(mid..this.len()) as *mut _ as *mut Self) },
        )
    }

    /// Returns a mutable iterator over the trits in this slice.
    ///
    /// Using this function is significantly faster than calling [`Trits::set`] in a loop and
    /// should be used where possible.
    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        self.as_raw_slice_mut().iter_mut()
    }
}

/// These functions are only implemented for trit slices with the [`T3B1`] encoding because only
/// the [`T3B1`] encoding has a representation compatible with a slice of `Tryte`s. If you find
/// yourself commonly needing to convert between trits and trytes, [`T3B1`] is the encoding to use.
impl Trits<T3B1> {
    /// Interpret this trit slice as a [`Tryte`] slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the length of the slice is not a multiple of `3`, or if the
    /// slice is not byte-aligned.
    pub fn as_trytes(&self) -> &[Tryte] {
        assert!(self.len() % 3 == 0, "Trit slice length must be a multiple of 3");
        unsafe { &*(self.as_i8_slice() as *const _ as *const _) }
    }

    /// Interpret this mutable trit slice as a mutable [`Tryte`] slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the length of the slice is not a multiple of `3`, or if the
    /// slice is not byte-aligned.
    pub fn as_trytes_mut(&mut self) -> &mut [Tryte] {
        assert!(self.len() % 3 == 0, "Trit slice length must be a multiple of 3");
        unsafe { &mut *(self.as_i8_slice_mut() as *mut _ as *mut _) }
    }
}

impl<T, U> cmp::PartialEq<Trits<U>> for Trits<T>
where
    T: RawEncoding + ?Sized,
    U: RawEncoding<Trit = T::Trit> + ?Sized,
{
    fn eq(&self, other: &Trits<U>) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<T, U> cmp::PartialOrd<Trits<U>> for Trits<T>
where
    T: RawEncoding + ?Sized,
    U: RawEncoding<Trit = T::Trit> + ?Sized,
    T::Trit: cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Trits<U>) -> Option<Ordering> {
        if self.len() != other.len() {
            return None;
        }

        for (a, b) in self.iter().zip(other.iter()) {
            match a.partial_cmp(&b) {
                Some(Ordering::Equal) => continue,
                other_order => return other_order,
            }
        }

        Some(Ordering::Equal)
    }
}

impl<'a, T: RawEncoding + ?Sized> fmt::Debug for &'a Trits<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Trits<{}> [", any::type_name::<T>())?;
        for (i, trit) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", trit)?;
        }
        write!(f, "]")
    }
}

// x

impl<T: RawEncoding + ?Sized> Index<usize> for Trits<T> {
    type Output = T::Trit;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of range").as_arbitrary_ref()
    }
}

// x..y

impl<T: RawEncoding + ?Sized> Index<Range<usize>> for Trits<T> {
    type Output = Self;
    fn index(&self, range: Range<usize>) -> &Self::Output {
        self.subslice(range)
    }
}
impl<T: RawEncoding + ?Sized> IndexMut<Range<usize>> for Trits<T> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        self.subslice_mut(range)
    }
}

// x..

impl<T: RawEncoding + ?Sized> Index<RangeFrom<usize>> for Trits<T> {
    type Output = Self;
    fn index(&self, range: RangeFrom<usize>) -> &Self::Output {
        self.subslice(range.start..self.len())
    }
}
impl<T: RawEncoding + ?Sized> IndexMut<RangeFrom<usize>> for Trits<T> {
    fn index_mut(&mut self, range: RangeFrom<usize>) -> &mut Self::Output {
        self.subslice_mut(range.start..self.len())
    }
}

// ..

impl<T: RawEncoding + ?Sized> Index<RangeFull> for Trits<T> {
    type Output = Self;
    fn index(&self, _range: RangeFull) -> &Self::Output {
        self
    }
}
impl<T: RawEncoding + ?Sized> IndexMut<RangeFull> for Trits<T> {
    fn index_mut(&mut self, _range: RangeFull) -> &mut Self::Output {
        self
    }
}

// x..=y

impl<T: RawEncoding + ?Sized> Index<RangeInclusive<usize>> for Trits<T> {
    type Output = Self;
    fn index(&self, range: RangeInclusive<usize>) -> &Self::Output {
        self.subslice(*range.start()..*range.end() + 1)
    }
}
impl<T: RawEncoding + ?Sized> IndexMut<RangeInclusive<usize>> for Trits<T> {
    fn index_mut(&mut self, range: RangeInclusive<usize>) -> &mut Self::Output {
        self.subslice_mut(*range.start()..*range.end() + 1)
    }
}

// ..y

impl<T: RawEncoding + ?Sized> Index<RangeTo<usize>> for Trits<T> {
    type Output = Self;
    fn index(&self, range: RangeTo<usize>) -> &Self::Output {
        self.subslice(0..range.end)
    }
}
impl<T: RawEncoding + ?Sized> IndexMut<RangeTo<usize>> for Trits<T> {
    fn index_mut(&mut self, range: RangeTo<usize>) -> &mut Self::Output {
        self.subslice_mut(0..range.end)
    }
}

// ..=y

impl<T: RawEncoding + ?Sized> Index<RangeToInclusive<usize>> for Trits<T> {
    type Output = Self;
    fn index(&self, range: RangeToInclusive<usize>) -> &Self::Output {
        self.subslice(0..range.end + 1)
    }
}
impl<T: RawEncoding + ?Sized> IndexMut<RangeToInclusive<usize>> for Trits<T> {
    fn index_mut(&mut self, range: RangeToInclusive<usize>) -> &mut Self::Output {
        self.subslice_mut(0..range.end + 1)
    }
}

impl<T: RawEncoding + ?Sized> ToOwned for Trits<T> {
    type Owned = TritBuf<T::Buf>;

    fn to_owned(&self) -> Self::Owned {
        self.to_buf()
    }
}

impl<T: RawEncoding + ?Sized> fmt::Display for Trits<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, t) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", t)?;
        }
        write!(f, "]")
    }
}

/// A buffer containing trits.
///
/// This type is roughly analogous to [`Vec`] or [`String`]. It supports pushing and popping trits
/// and dereferences to [`Trits`]. It may be borrowed as a trit slice, either mutably or immutably.
#[derive(Clone)]
#[repr(transparent)]
pub struct TritBuf<T: RawEncodingBuf = T1B1Buf<Btrit>>(T);

impl<T: RawEncodingBuf> TritBuf<T> {
    /// Create a new empty [`TritBuf`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new empty [`TritBuf`], backed by the given capacity, `cap`. The resulting
    /// [`TritBuf`] will contain at least enough space to contain `cap` trits without needing to
    /// reallocate.
    fn with_capacity(_cap: usize) -> Self {
        // TODO: Allocate capacity
        Self::new()
    }

    /// Create a new [`TritBuf`] of the given length, filled with copies of the provided trit.
    pub fn filled(len: usize, trit: <T::Slice as RawEncoding>::Trit) -> Self {
        let mut this = Self::with_capacity(len);
        for _ in 0..len {
            this.push(trit);
        }
        this
    }

    /// Create a new [`TritBuf`] of the given length, filled with zero trit.
    pub fn zeros(len: usize) -> Self {
        Self::filled(len, <T::Slice as RawEncoding>::Trit::zero())
    }

    /// Create a new [`TritBuf`] containing the trits from the given slice of trits.
    pub fn from_trits(trits: &[<T::Slice as RawEncoding>::Trit]) -> Self {
        Self(T::from_trits(trits))
    }

    /// Push a trit to the back of this [`TritBuf`].
    pub fn push(&mut self, trit: <T::Slice as RawEncoding>::Trit) {
        self.0.push(trit);
    }

    /// Pop a trit from the back of this [`TritBuf`], returning it if successful.
    pub fn pop(&mut self) -> Option<<T::Slice as RawEncoding>::Trit> {
        self.0.pop()
    }

    /// Extracts a trit slice containing the data within this buffer.
    ///
    /// Note that [`TritBuf`] dereferences to `Trits` anyway, so it's usually sufficient to take
    /// a reference to [`TritBuf`] or to just call `&Trits` methods on it rather than explicitly
    /// calling this method first.
    pub fn as_slice(&self) -> &Trits<T::Slice> {
        unsafe { &*(self.0.as_slice() as *const T::Slice as *const Trits<T::Slice>) }
    }

    /// Extracts a mutable trit slice containing the data within this buffer.
    ///
    /// Note that [`TritBuf`] dereferences to `Trits` anyway, so it's usually sufficient to take
    /// a reference to [`TritBuf`] or to just call `&mut Trits` methods on it rather
    /// explicitly calling this method first.
    pub fn as_slice_mut(&mut self) -> &mut Trits<T::Slice> {
        unsafe { &mut *(self.0.as_slice_mut() as *mut T::Slice as *mut Trits<T::Slice>) }
    }
}

impl TritBuf<T3B1Buf> {
    /// Pad the trit buffer with [`Btrit::Zero`] until the buffer's length is a multiple of 3.
    ///
    /// This method is often used in conjunction with [`Trites::as_trytes`].
    pub fn pad_zeros(&mut self) {
        while self.len() % 3 != 0 {
            self.push(Btrit::Zero);
        }
    }

    /// Pad the trit buffer with [`Btrit::Zero`] until the buffer's length is a multiple of 3.
    ///
    /// This method is often used in conjunction with [`Trites::as_trytes`].
    pub fn padded_zeros(mut self) -> Self {
        self.pad_zeros();
        self
    }
}

impl<T: RawEncodingBuf> Neg for TritBuf<T>
where
    T::Slice: RawEncoding<Trit = Btrit>,
{
    type Output = Self;

    fn neg(mut self) -> Self {
        self.negate();
        self
    }
}

impl<T: RawEncodingBuf> TritBuf<T>
where
    T::Slice: RawEncoding<Trit = Btrit>,
{
    /// Create a new [`TritBuf`] containing the trits given by the slice of i8s.
    pub fn from_i8s(trits: &[i8]) -> Result<Self, <Btrit as TryFrom<i8>>::Error> {
        trits.iter().map(|x| Btrit::try_from(*x)).collect()
    }
}

impl<T: RawEncodingBuf> TritBuf<T>
where
    T::Slice: RawEncoding<Trit = Utrit>,
{
    /// Create a new [`TritBuf`] containing the trits given by the slice of u8s.
    pub fn from_u8s(trits: &[u8]) -> Result<Self, <Btrit as TryFrom<u8>>::Error> {
        trits.iter().map(|x| Utrit::try_from(*x)).collect()
    }
}

impl<T: RawEncodingBuf> Default for TritBuf<T> {
    fn default() -> Self {
        Self(T::new())
    }
}

impl<T> TritBuf<T1B1Buf<T>>
where
    T: Trit,
    T::Target: Trit,
{
    /// Transform this [`TritBuf`] into a shifted representation. If the buffer contains
    /// balanced trits ([`Btrit`]), the returned buffer will contain unbalanced trits ([`Utrit`]).
    pub fn into_shifted(self) -> TritBuf<T1B1Buf<<T as ShiftTernary>::Target>> {
        TritBuf(self.0.into_shifted())
    }
}

impl<T: RawEncodingBuf, U: RawEncodingBuf> PartialEq<TritBuf<U>> for TritBuf<T>
where
    T::Slice: RawEncoding,
    U::Slice: RawEncoding<Trit = <T::Slice as RawEncoding>::Trit>,
{
    fn eq(&self, other: &TritBuf<U>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: RawEncodingBuf> Deref for TritBuf<T> {
    type Target = Trits<T::Slice>;

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: RawEncodingBuf> DerefMut for TritBuf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T: RawEncodingBuf> FromIterator<<T::Slice as RawEncoding>::Trit> for TritBuf<T> {
    fn from_iter<I: IntoIterator<Item = <T::Slice as RawEncoding>::Trit>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut this = Self::with_capacity(iter.size_hint().0);
        for trit in iter {
            this.push(trit);
        }
        this
    }
}

impl<T> hash::Hash for TritBuf<T>
where
    T: RawEncodingBuf,
    T::Slice: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
    }
}

impl<T: RawEncodingBuf> fmt::Debug for TritBuf<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TritBuf<{}> [", any::type_name::<T>())?;
        for (i, trit) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", trit)?;
        }
        write!(f, "]")
    }
}

impl<T: RawEncodingBuf> fmt::Display for TritBuf<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_slice())
    }
}

impl<T: RawEncodingBuf> Borrow<Trits<T::Slice>> for TritBuf<T> {
    fn borrow(&self) -> &Trits<T::Slice> {
        self.as_slice()
    }
}

impl<T: RawEncodingBuf> BorrowMut<Trits<T::Slice>> for TritBuf<T> {
    fn borrow_mut(&mut self) -> &mut Trits<T::Slice> {
        self.as_slice_mut()
    }
}
