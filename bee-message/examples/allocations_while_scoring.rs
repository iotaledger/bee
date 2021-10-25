// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This script aims at counting the number of allocations that are performed when we score the Proof of Work of a
//! message by wrapping `GlobalAlloc`. Ideally, this method should not allocate at all, which would lead to a better
//! performance.
//!
//! The code was adapted from: https://kanejaku.org/posts/2021/01/2021-01-27/ (CC-BY 4.0)

use bee_message::prelude::*;
use bee_packable::PackableExt;
use bee_pow::{
    providers::{miner::MinerBuilder, NonceProviderBuilder},
    score::PoWScorer,
};
use bee_test::rand::parents::rand_parents;

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
    let message = MessageBuilder::new()
        .with_network_id(0)
        .with_parents(rand_parents())
        .with_nonce_provider(MinerBuilder::new().with_num_workers(num_cpus::get()).finish(), 10000f64)
        .finish()
        .unwrap();

    let message_bytes = message.pack_to_vec();

    let before_count = ALLOCATED.load(SeqCst);
    let _score = PoWScorer::new().score(&message_bytes);
    let after_count = ALLOCATED.load(SeqCst);

    println!("Number of allocations: {}", after_count - before_count);
}
