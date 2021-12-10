// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::task;

use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = bee_trace::subscriber::build().with_console_layer().init().unwrap();

    println!("Running tasks indefinitely...");
    let task1 = task::spawn(run_tasks(500));
    let task2 = task::spawn(run_tasks(200));
    let _ = tokio::join!(task1, task2);

    Ok(())
}

async fn run_tasks(busy_per_sec: u64) {
    loop {
        let idle_per_sec = 1000 - busy_per_sec;

        let _ = tokio::try_join!(
            task::spawn(full_duration_task(busy_per_sec, idle_per_sec)),
            task::spawn(half_duration_task(busy_per_sec, idle_per_sec)),
        );
    }
}

async fn full_duration_task(busy: u64, idle: u64) {
    let half_busy = busy / 2;
    let idle_per_sec = 1000 - half_busy;
    half_duration_task(half_busy, idle_per_sec).await;

    std::thread::sleep(Duration::from_millis(busy));
    tokio::time::sleep(Duration::from_millis(idle)).await;
}

async fn half_duration_task(busy: u64, idle: u64) {
    let half_busy = busy / 2;
    let idle_per_sec = 1000 - half_busy;
    quarter_duration_task(half_busy, idle_per_sec).await;

    std::thread::sleep(Duration::from_millis(busy));
    tokio::time::sleep(Duration::from_millis(idle)).await;
}

async fn quarter_duration_task(busy: u64, idle: u64) {
    std::thread::sleep(Duration::from_millis(busy));
    tokio::time::sleep(Duration::from_millis(idle)).await;
}
