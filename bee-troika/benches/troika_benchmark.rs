#[macro_use]
extern crate criterion;
extern crate rand;

use bee_troika::Ftroika;
use bee_troika::Troika;

use criterion::Criterion;
use rand::{thread_rng, Rng};

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
