/*
 * Copyright (C) 2019 Yu-Wei Wu
 * All Rights Reserved.
 * This is free software; you can redistribute it and/or modify it under the
 * terms of the MIT license. A copy of the license can be found in the file
 * "LICENSE" at the root of this distribution.
 */

#[macro_use]
extern crate criterion;
extern crate rand;
use rand::{thread_rng, Rng};

use criterion::Criterion;
use troika_rust::Ftroika;
use troika_rust::Troika;

fn basic_troika() {
    let mut troika = Troika::default();
    let mut input = [0u8; 8019];
    let mut output = [0u8; 243];
    let mut rng = thread_rng();

    for trit in input.iter_mut() {
        *trit = rng.gen_range(0, 3);
    }

    troika.absorb(&input);
    troika.squeeze(&mut output);
}

fn basic_ftroika() {
    let mut ftroika = Ftroika::default();
    let mut input = [0u8; 8019];
    let mut output = [0u8; 243];
    let mut rng = thread_rng();

    for trit in input.iter_mut() {
        *trit = rng.gen_range(0, 3);
    }

    ftroika.absorb(&input);
    ftroika.finalize();
    ftroika.squeeze(&mut output);
}

fn ftroika_benchmark(c: &mut Criterion) {
    c.bench_function("Ftroika with input of 8019 trits", |b| b.iter(|| basic_ftroika()));
}

fn troika_benchmark(c: &mut Criterion) {
    c.bench_function("Troika with input of 8019 trits", |b| b.iter(|| basic_troika()));
}

criterion_group!(benches, ftroika_benchmark, troika_benchmark);
criterion_main!(benches);
