// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A batched version of the `CurlP` hash.

mod bct;
mod bct_curlp;

#[allow(deprecated)]
use crate::ternary::sponge::{CurlP, CurlPRounds, Sponge, HASH_LENGTH};

use bct::{BcTrit, BcTritArr, BcTritBuf};
use bct_curlp::BctCurlP;

use bee_ternary::{
    raw::{RawEncoding, RawEncodingBuf},
    Btrit, T1B1Buf, TritBuf,
};

/// The number of inputs that can be processed in a single batch.
pub const BATCH_SIZE: usize = 8 * std::mem::size_of::<usize>();
const HIGH_BITS: usize = usize::max_value();

#[allow(deprecated)]
struct CurlPLegacy(CurlP);

/// A hasher that can process several inputs at the same time in batches.
///
/// This hasher works by interleaving the trits of the inputs in each batch and hashing this
/// interleaved representation. It is also able to fall back to the regular CurlP algorithm if
/// required.
pub struct BatchHasher<B: RawEncodingBuf> {
    /// The trits of the inputs before being interleaved.
    trit_inputs: Vec<TritBuf<B>>,
    /// An interleaved representation of the input trits.
    bct_inputs: BcTritBuf,
    /// An interleaved representation of the output trits.
    bct_hashes: BcTritArr<HASH_LENGTH>,
    /// A buffer for demultiplexing.
    buf_demux: TritBuf,
    /// The CurlP hasher for binary coded trits.
    bct_curlp: BctCurlP,
    /// The regular CurlP hasher.
    curlp: CurlPLegacy,
}

impl<B> BatchHasher<B>
where
    B: RawEncodingBuf,
    B::Slice: RawEncoding<Trit = Btrit>,
{
    /// Create a new hasher.
    ///
    /// It requires the length of the input, the length of the output hash and the number of
    /// rounds.
    #[deprecated(note = "Only CurlP with 81 rounds will be supported in future versions.")]
    #[allow(deprecated)]
    pub fn new(input_length: usize, rounds: CurlPRounds) -> Self {
        Self {
            trit_inputs: Vec::with_capacity(BATCH_SIZE),
            bct_inputs: BcTritBuf::zeros(input_length),
            bct_hashes: BcTritArr::<HASH_LENGTH>::zeros(),
            buf_demux: TritBuf::zeros(HASH_LENGTH),
            bct_curlp: BctCurlP::new(rounds),
            curlp: CurlPLegacy(CurlP::new(rounds)),
        }
    }

    /// Add a new input to the batch.
    ///
    /// It panics if the size of the batch exceeds `BATCH_SIZE` or if `input.len()` is not equal to
    /// the `input_length` parameter of the constructor.
    pub fn add(&mut self, input: TritBuf<B>) {
        assert!(self.trit_inputs.len() < BATCH_SIZE, "Batch is full.");
        assert_eq!(input.len(), self.bct_inputs.len(), "Input has an incorrect size.");
        self.trit_inputs.push(input);
    }

    /// Return the length of the current batch.
    pub fn len(&self) -> usize {
        self.trit_inputs.len()
    }

    /// Check if the current batch is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Multiplex or interleave the input trits in the bash.
    ///
    /// Before doing the actual interleaving, each trit is encoded as two bits which are usually
    /// refered as the low and high bit.
    ///
    /// | Trit | Low bit | High bit |
    /// |------|---------|----------|
    /// |  -1  |    1    |     0    |
    /// |   0  |    0    |     1    |
    /// |   1  |    1    |     1    |
    ///
    /// Then the low and high bits are interleaved into two vectors of integers. Each integer has
    /// size `BATCH_SIZE` and there are `input_length` integers in each vector.  This means that
    /// the low and high bits of the transaction number `N` in the batch are stored in the position
    /// `N` of each integer.
    ///
    /// This step works correctly even if there are less than `BATCH_SIZE` inputs.
    fn mux(&mut self) {
        let count = self.trit_inputs.len();
        for i in 0..self.bct_inputs.len() {
            // This is safe because `i < self.bct_inputs.len()`.
            let BcTrit(lo, hi) = unsafe { self.bct_inputs.get_unchecked_mut(i) };

            for j in 0..count {
                // this is safe because `j < self.trit_inputs.len()` and
                // `i < self.trit_inputs[j].len()` (the `add` method guarantees that all the inputs
                // have the same length as `self.trit_inputs`).
                match unsafe { self.trit_inputs.get_unchecked(j).get_unchecked(i) } {
                    Btrit::NegOne => *lo |= 1 << j,
                    Btrit::PlusOne => *hi |= 1 << j,
                    Btrit::Zero => {
                        *lo |= 1 << j;
                        *hi |= 1 << j;
                    }
                }
            }
        }
    }

    /// Demultiplex the bits of the output to obtain the hash of the input with a specific index.
    ///
    /// This is the inverse of the `mux` function, but it is applied over the vector with the
    /// binary encoding of the output hashes. Each pair of low and high bits in the `bct_hashes`
    /// field is decoded into a trit using the same convention as the `mux` step with an additional
    /// rule for the `(0, 0)` pair of bits which is mapped to the `0` trit.
    fn demux(&mut self, index: usize) -> TritBuf {
        for (bc_trit, btrit) in self.bct_hashes.iter().zip(self.buf_demux.iter_mut()) {
            let lo = (bc_trit.lo() >> index) & 1;
            let hi = (bc_trit.hi() >> index) & 1;

            *btrit = match (lo, hi) {
                (1, 0) => Btrit::NegOne,
                (0, 1) => Btrit::PlusOne,
                // This can only be `(0, 0)` or `(1, 1)`.
                _ => Btrit::Zero,
            };
        }

        self.buf_demux.clone()
    }

    /// Hash the received inputs using the batched version of CurlP.
    ///
    /// This function also takes care of cleaning the buffers of the struct and resetting the
    /// batched CurlP hasher so it can be called at any time.
    pub fn hash_batched(&mut self) -> impl Iterator<Item = TritBuf> + '_ {
        let total = self.trit_inputs.len();
        // Reset batched CurlP hasher.
        self.bct_curlp.reset();
        // Multiplex the trits in `trits` and dump them into `inputs`
        self.mux();
        // Do the regular sponge steps.
        self.bct_curlp.absorb(&self.bct_inputs);
        self.bct_curlp.squeeze_into(&mut self.bct_hashes);
        // Clear the `trits` buffer to allow receiving a new batch.
        self.trit_inputs.clear();
        // Fill the `inputs` buffer with zeros.
        self.bct_inputs.fill(0);
        // Return an iterator for the output hashes.
        BatchedHashes {
            hasher: self,
            range: 0..total,
        }
    }

    /// Hash the received inputs using the regular version of CurlP.
    ///
    /// In particular this function does not use the `bct_inputs` and `bct_hashes` buffers, takes
    /// care of cleaning the `trit_inputs` buffer and resets the regular CurlP hasher so it can be
    /// called at any time.
    pub fn hash_unbatched(&mut self) -> impl Iterator<Item = TritBuf> + '_ {
        self.curlp.0.reset();
        UnbatchedHashes {
            curl: &mut self.curlp,
            trit_inputs: self.trit_inputs.drain(..),
        }
    }
}

/// A helper iterator type for the output of the `hash_batched` method.
struct BatchedHashes<'a, B: RawEncodingBuf> {
    hasher: &'a mut BatchHasher<B>,
    range: std::ops::Range<usize>,
}

impl<'a, B> Iterator for BatchedHashes<'a, B>
where
    B: RawEncodingBuf,
    B::Slice: RawEncoding<Trit = Btrit>,
{
    type Item = TritBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;
        Some(self.hasher.demux(index))
    }
}

/// A helper iterator type for the output of the `hash_unbatched` method.
struct UnbatchedHashes<'a, B: RawEncodingBuf> {
    curl: &'a mut CurlPLegacy,
    trit_inputs: std::vec::Drain<'a, TritBuf<B>>,
}

impl<'a, B> Iterator for UnbatchedHashes<'a, B>
where
    B: RawEncodingBuf,
    B::Slice: RawEncoding<Trit = Btrit>,
{
    type Item = TritBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.trit_inputs.next()?;
        // FIXME: Could we make regular `CurlP` generic too?`
        Some(self.curl.0.digest(&buf.encode::<T1B1Buf>()).unwrap())
    }
}
