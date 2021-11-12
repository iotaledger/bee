// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains utilities to score Proof of Work.

use bee_ternary::{b1t6, Btrit, T1B1Buf, TritBuf, Trits, T1B1};

use crypto::hashes::{
    blake2b::Blake2b256,
    ternary::{curl_p::CurlP, HASH_LENGTH},
    Digest,
};
use bee_crypto::ternary::sponge::{Sponge, UnrolledCurlP81};

/// Encapsulates the different steps that are used for scoring Proof of Work.
pub struct PoWScorer {
    blake2b: Blake2b256,
    pow_input: TritBuf<T1B1Buf>,
    curl: UnrolledCurlP81,
}

impl PoWScorer {
    /// Creates an new `PoWScorer` that holds the required hash functions as internal state.
    pub fn new() -> Self {
        Self {
            blake2b: Blake2b256::new(),
            pow_input: TritBuf::<T1B1Buf>::with_capacity(HASH_LENGTH),
            curl: UnrolledCurlP81::new(),
        }
    }

    /// Returns the Proof of Work hash of given bytes.
    /// Panic: expects at least 8 bytes.
    pub fn hash(&mut self, bytes: &[u8]) -> TritBuf<T1B1Buf> {
        debug_assert!(bytes.len() >= std::mem::size_of::<u8>());

        // Compute Blake2b-256 hash of the message, excluding the nonce.
        let length = bytes.len() - std::mem::size_of::<u64>();
        let (head, tail) = bytes.split_at(length);
        self.blake2b.update(head);
        let pow_digest = self.blake2b.finalize_reset();

        // Encode message as trits
        self.pow_input.clear();
        b1t6::encode::<T1B1Buf>(&pow_digest)
            .iter()
            .for_each(|t| self.pow_input.push(t));
        b1t6::encode::<T1B1Buf>(tail)
            .iter()
            .for_each(|t| self.pow_input.push(t));

        // Pad to 243 trits
        self.pow_input.push(Btrit::Zero);
        self.pow_input.push(Btrit::Zero);
        self.pow_input.push(Btrit::Zero);

        // TODO: Consider using an output buffer here, for example by using the `Sponge` mechanism?
        self.curl.digest(self.pow_input.as_slice()).unwrap()
    }

    /// Computes the Proof of Work score of given bytes.
    /// Panic: expects at least 8 bytes.
    pub fn score(&mut self, bytes: &[u8]) -> f64 {
        pow_score_for_hash(&self.hash(bytes), bytes.len())
    }
}

/// Returns the Proof of Work hash of given bytes.
/// Panic: expects at least 8 bytes.
#[deprecated(note = "Use `PoWScorer::hash` instead.")]
pub fn pow_hash(bytes: &[u8]) -> TritBuf<T1B1Buf> {
    PoWScorer::new().hash(bytes)
}

/// Computes the Proof of Work score of given bytes.
/// Panic: expects at least 8 bytes.
#[deprecated(note = "Use `PoWScorer::score` instead.")]
pub fn compute_pow_score(bytes: &[u8]) -> f64 {
    debug_assert!(bytes.len() >= std::mem::size_of::<u8>());

    #[allow(deprecated)]
    pow_score_for_hash(&pow_hash(bytes), bytes.len())
}

/// Returns the number of trailing zeros of a Proof of Work hash.
pub fn count_trailing_zeros(pow_hash: &Trits<T1B1>) -> usize {
    pow_hash.iter().rev().take_while(|t| *t == Btrit::Zero).count()
}

/// Returns the Proof of Work score of a Proof of Work hash.
pub fn pow_score_for_hash(pow_hash: &Trits<T1B1>, len: usize) -> f64 {
    3u128.pow(count_trailing_zeros(pow_hash) as u32) as f64 / len as f64
}

impl Default for PoWScorer {
    fn default() -> Self {
        Self::new()
    }
}
