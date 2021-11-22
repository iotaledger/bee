// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This script aims at counting the number of allocations that are performed when we score the Proof of Work of a
//! message by wrapping `GlobalAlloc`. Ideally, this method should not allocate at all, which would lead to a better
//! performance.
//!
//! The code was adapted from: https://kanejaku.org/posts/2021/01/2021-01-27/ (CC-BY 4.0)

use bee_common::packable::Packable;
use bee_pow::{
    providers::{miner::MinerBuilder, NonceProvider, NonceProviderBuilder},
    score::PoWScorer,
};
use bee_test::rand::message::rand_message;

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

struct CheckAlloc;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CheckAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(1, SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static A: CheckAlloc = CheckAlloc;

fn main() {
    let message = rand_message();

    let msg_bytes = message.pack_new();

    let before_count = ALLOCATED.load(SeqCst);
    let _score = PoWScorer::new().score(&msg_bytes);
    let after_count = ALLOCATED.load(SeqCst);

    println!("Number of allocations for scoring: {}", after_count - before_count);

    let miner = MinerBuilder::default().with_num_workers(num_cpus::get()).finish();
    let minimum_pow_score = 4000.0;
    let before_count = ALLOCATED.load(SeqCst);
    let _nonce = miner.nonce(&msg_bytes[0..msg_bytes.len() - std::mem::size_of::<u64>()], minimum_pow_score);
    let after_count = ALLOCATED.load(SeqCst);

    println!("Number of allocations for finding a nonce: {}", after_count - before_count);
}
