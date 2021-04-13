// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains a nonce provider that mine nonces.

use crate::providers::{NonceProvider, NonceProviderBuilder};

use bee_crypto::ternary::{
    sponge::{BatchHasher, CurlPRounds, BATCH_SIZE},
    HASH_LENGTH,
};
use bee_ternary::{b1t6, Btrit, T1B1Buf, TritBuf};

use crypto::hashes::{blake2b::Blake2b256, Digest};
use thiserror::Error;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

const DEFAULT_NUM_WORKERS: usize = 1;
// Precomputed natural logarithm of 3 for performance reasons.
// See https://oeis.org/A002391.
const LN_3: f64 = 1.098_612_288_668_109;

/// Errors occurring when computing nonces with the `Miner` nonce provider.
#[derive(Error, Debug)]
pub enum Error {
    /// The worker has been cancelled.
    #[error("The worker has been cancelled.")]
    Cancelled,
    /// Invalid proof of work score.
    #[error("Invalid proof of work score {0}, requiring {} trailing zeros.")]
    InvalidPowScore(f64, usize),
}

/// A type to cancel the `Miner` nonce provider to abort operations.
#[derive(Default, Clone)]
pub struct MinerCancel(Arc<AtomicBool>);

impl MinerCancel {
    /// Creates a new `MinerCancel`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cancels the `Miner` nonce provider.
    pub fn trigger(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Checks if cancellation has been triggered.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Reset the cancel flag.
    fn reset(&self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

/// Builder for the `Miner` nonce provider.
#[derive(Default)]
pub struct MinerBuilder {
    num_workers: Option<usize>,
    cancel: Option<MinerCancel>,
}

impl MinerBuilder {
    /// Sets the desired number of workers for the `Miner` nonce provider.
    pub fn with_num_workers(mut self, num_workers: usize) -> Self {
        self.num_workers.replace(num_workers);
        self
    }

    /// Sets a `MinerCancel to abort the `Miner` nonce provider.
    pub fn with_cancel(mut self, cancel: MinerCancel) -> Self {
        self.cancel.replace(cancel);
        self
    }
}

impl NonceProviderBuilder for MinerBuilder {
    type Provider = Miner;

    fn finish(self) -> Miner {
        Miner {
            num_workers: self.num_workers.unwrap_or(DEFAULT_NUM_WORKERS),
            cancel: self.cancel.unwrap_or_else(MinerCancel::new),
        }
    }
}

/// A nonce provider that mine nonces.
pub struct Miner {
    num_workers: usize,
    cancel: MinerCancel,
}

impl Miner {
    fn worker(
        cancel: MinerCancel,
        pow_digest: TritBuf<T1B1Buf>,
        start_nonce: u64,
        target_zeros: usize,
    ) -> Result<u64, Error> {
        let mut nonce = start_nonce;
        let mut hasher = BatchHasher::<T1B1Buf>::new(HASH_LENGTH, CurlPRounds::Rounds81);
        let mut buffers = Vec::<TritBuf<T1B1Buf>>::with_capacity(BATCH_SIZE);

        for _ in 0..BATCH_SIZE {
            let mut buffer = TritBuf::<T1B1Buf>::zeros(HASH_LENGTH);
            buffer[..pow_digest.len()].copy_from(&pow_digest);
            buffers.push(buffer);
        }

        while !cancel.is_cancelled() {
            for (i, buffer) in buffers.iter_mut().enumerate() {
                let nonce_trits = b1t6::encode::<T1B1Buf>(&(nonce + i as u64).to_le_bytes());
                buffer[pow_digest.len()..pow_digest.len() + nonce_trits.len()].copy_from(&nonce_trits);
                hasher.add(buffer.clone());
            }

            for (i, hash) in hasher.hash_batched().enumerate() {
                let trailing_zeros = hash.iter().rev().take_while(|t| *t == Btrit::Zero).count();

                if trailing_zeros >= target_zeros {
                    cancel.trigger();
                    return Ok(nonce + i as u64);
                }
            }

            nonce += BATCH_SIZE as u64;
        }

        Err(Error::Cancelled)
    }
}

impl NonceProvider for Miner {
    type Builder = MinerBuilder;
    type Error = Error;

    fn nonce(&self, bytes: &[u8], target_score: f64) -> Result<u64, Self::Error> {
        self.cancel.reset();

        let mut nonce = 0;
        let mut pow_digest = TritBuf::<T1B1Buf>::new();
        let target_zeros =
            (((bytes.len() + std::mem::size_of::<u64>()) as f64 * target_score).ln() / LN_3).ceil() as usize;

        if target_zeros > HASH_LENGTH {
            return Err(Self::Error::InvalidPowScore(target_score, target_zeros));
        }

        let worker_width = u64::MAX / self.num_workers as u64;
        let mut workers = Vec::with_capacity(self.num_workers);
        let hash = Blake2b256::digest(&bytes);

        b1t6::encode::<T1B1Buf>(&hash).iter().for_each(|t| pow_digest.push(t));

        for i in 0..self.num_workers {
            let start_nonce = i as u64 * worker_width;
            let _cancel = self.cancel.clone();
            let _pow_digest = pow_digest.clone();

            workers.push(thread::spawn(move || {
                Miner::worker(_cancel, _pow_digest, start_nonce, target_zeros)
            }));
        }

        for worker in workers {
            nonce = match worker.join().unwrap() {
                Ok(nonce) => nonce,
                Err(_) => continue,
            }
        }

        Ok(nonce)
    }
}
