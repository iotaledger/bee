// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::providers::{Provider, ProviderBuilder};

use bee_common::b1t6;
use bee_crypto::ternary::{
    sponge::{BatchHasher, CurlPRounds, BATCH_SIZE},
    HASH_LENGTH,
};
use bee_ternary::{Btrit, T1B1Buf, TritBuf};

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use thiserror::Error;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

const DEFAULT_NUM_WORKERS: usize = 1;
/// https://oeis.org/A002391
const LN_3: f64 = 1.098_612_288_668_109_8;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The worker has been cancelled.")]
    Cancelled,
}

#[derive(Default)]
pub struct MinerBuilder {
    num_workers: Option<usize>,
}

impl MinerBuilder {
    pub fn with_num_workers(mut self, num_workers: usize) -> Self {
        self.num_workers = Some(num_workers);
        self
    }
}

impl ProviderBuilder for MinerBuilder {
    type Provider = Miner;

    fn new() -> Self {
        Self::default()
    }

    fn finish(self) -> Miner {
        Miner {
            num_workers: self.num_workers.unwrap_or(DEFAULT_NUM_WORKERS),
        }
    }
}

pub struct Miner {
    num_workers: usize,
}

impl Miner {
    fn worker(
        done: Arc<AtomicBool>,
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

        while !done.load(Ordering::Relaxed) {
            for (i, buffer) in buffers.iter_mut().enumerate().take(BATCH_SIZE) {
                let nonce_trits = b1t6::encode(&(nonce + i as u64).to_le_bytes());
                buffer[pow_digest.len()..pow_digest.len() + nonce_trits.len()].copy_from(&nonce_trits);
                hasher.add(buffer.clone());
            }

            for (i, hash) in hasher.hash_batched().enumerate() {
                let trainling_zeros = hash.iter().rev().take_while(|t| *t == Btrit::Zero).count();

                if trainling_zeros >= target_zeros {
                    done.store(true, Ordering::Relaxed);
                    return Ok(nonce + i as u64);
                }
            }

            nonce += BATCH_SIZE as u64;
        }

        Err(Error::Cancelled)
    }
}

impl Provider for Miner {
    type Builder = MinerBuilder;
    type Error = Error;

    fn nonce(&self, bytes: &[u8], target_score: f64) -> Result<u64, Self::Error> {
        let mut nonce = 0;
        let mut blake = VarBlake2b::new(32).unwrap();
        let mut pow_digest = TritBuf::<T1B1Buf>::new();

        blake.update(&bytes);
        blake.finalize_variable_reset(|hash| b1t6::encode(&hash).iter().for_each(|t| pow_digest.push(t)));

        let target_zeros =
            (((bytes.len() + std::mem::size_of::<u64>()) as f64 * target_score).ln() / LN_3).ceil() as usize;
        let worker_width = u64::MAX / self.num_workers as u64;

        let done = Arc::new(AtomicBool::new(false));
        let mut workers = Vec::with_capacity(self.num_workers);

        for i in 0..self.num_workers {
            let start_nonce = i as u64 * worker_width;
            let _done = done.clone();
            let _pow_digest = pow_digest.clone();

            workers.push(thread::spawn(move || {
                Miner::worker(_done, _pow_digest, start_nonce, target_zeros)
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
