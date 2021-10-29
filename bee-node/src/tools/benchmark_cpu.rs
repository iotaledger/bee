use std::{sync::{Arc, atomic::{AtomicU64, Ordering}}, thread::{self}, time::{self, Duration}};

use bee_crypto::ternary::{sponge::{BATCH_SIZE, CurlP, CurlPRounds, Sponge}};
use bee_pow::providers::miner::MinerCancel;
use bee_ternary::{T1B1Buf, TritBuf, b1t6::{self}};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchmarkCPUError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug, StructOpt)]
pub struct BenchmarkCPUTool {
    threads: Option<usize>
}

pub fn exec(tool: &BenchmarkCPUTool) {
    let threads = match tool.threads {
        Some(threads) => threads,
        None => num_cpus::get()
    };
    let duration = Duration::from_secs(60);

    println!("Benchmarking CPU with {} threads", threads);

    let cancel = MinerCancel::new();
    let cancel_2 = cancel.clone();
    let cancel_3 = cancel.clone();
    let counter = Arc::new(AtomicU64::new(0));
    let counter_2 = counter.clone();

    let time_start = std::time::Instant::now();

    let pow_digest: [u8;32] = rand::random();

    let mut workers = Vec::with_capacity(threads+2);

    //Stop if the timeout has exceeded
    let time_thread = thread::spawn(move || {
        std::thread::sleep(duration);
        cancel.trigger();
    });

    let process_thread = thread::spawn(move || {
        while !cancel_2.is_cancelled() {
            std::thread::sleep(Duration::from_secs(2));
            
            let elapsed = time_start.elapsed();
			let (percentage, remaining) = estimate_remaining_time(time_start, elapsed.as_millis() as i64, duration.as_millis() as i64);
            let megahashes_per_second = counter.load(Ordering::Relaxed) as f64 / (elapsed.as_secs_f64() * 1_000_000 as f64);
            println!("Average CPU speed: {:.2}MH/s ({} thread(s), {:.2}%. {:.2?} left...)", megahashes_per_second, threads, percentage, remaining);
        }
    });

    let worker_width = u64::MAX / threads as u64;
    for i in 0..threads {
        let start_nonce = i as u64 * worker_width;
        let benchmark_cancel = cancel_3.clone();
        let benchmark_counter = counter_2.clone();
        let _pow_digest = pow_digest.clone();

        workers.push(thread::spawn(move || {
            cpu_benchmark_worker(&pow_digest, start_nonce, benchmark_cancel, benchmark_counter)
        }));
    }

    workers.push(process_thread);
    workers.push(time_thread);

    for worker in workers {
        worker.join().expect("Couldn't stop thread");
    }

    let megahashes_per_second = counter_2.load(Ordering::Relaxed) as f64 / (duration.as_secs_f64() * 1_000_000 as f64);
    println!("Average CPU speed: {:.2}MH/s ({} thread(s), took {:.2?})", megahashes_per_second, threads, duration);
}

fn cpu_benchmark_worker(_pow_digest: &[u8], start_nonce: u64, cancel: MinerCancel, counter: Arc<AtomicU64>) {
    let mut pow_digest = TritBuf::<T1B1Buf>::new();
    b1t6::encode::<T1B1Buf>(&_pow_digest).iter().for_each(|t| pow_digest.push(t));

    let mut nonce = start_nonce;
    let mut curlp = CurlP::new(CurlPRounds::Rounds81);
    let mut buffers = TritBuf::<T1B1Buf>::new();

    for i in 0..BATCH_SIZE {
        let nonce_trits = b1t6::encode::<T1B1Buf>(&(nonce + i as u64).to_le_bytes());
        buffers.append(&pow_digest);
        buffers.append(&nonce_trits);
    }

    while !cancel.is_cancelled() {
        curlp.reset();
        curlp.absorb(&*buffers).unwrap();

        counter.fetch_add(BATCH_SIZE as u64, Ordering::Release);

        nonce += BATCH_SIZE as u64;
    }
}

// Calculates the remaining time for a running operation and returns the finished percentage.
fn estimate_remaining_time(time_start: std::time::Instant, current: i64, total: i64) -> (f64, std::time::Duration) {
	let ratio = current as f64 / total as f64;
    let total_time = time::Duration::from_secs_f64(time_start.elapsed().as_secs_f64() / ratio);
    let time_now = std::time::Instant::now();
    if time_now > (time_start + total_time) {
        return (100.0, Duration::from_secs(0));
    }
    let remaining = (time_start + total_time).duration_since(time_now);
	return (ratio * 100.0, remaining)
}
