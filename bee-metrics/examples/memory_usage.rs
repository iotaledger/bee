// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_metrics::{metrics::MemoryUsage, serve_metrics, Registry};
use tokio::time::Duration;

async fn update_memory_usage(memory_usage: MemoryUsage) {
    let mut vec = Vec::new();

    loop {
        // Add 10KiB of data every second and update the metric.
        vec.extend_from_slice(&[0u8; 10240]);

        memory_usage.update().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() {
    let registry = Registry::default();

    let memory_usage = MemoryUsage::new(std::process::id());
    registry.register("memory_usage", "Resident set size", memory_usage.clone());

    let handle = { tokio::spawn(async move { update_memory_usage(memory_usage).await }) };

    let serve_fut = serve_metrics("0.0.0.0:3030".parse().unwrap(), registry);

    let (serve_res, handle_res) = tokio::join!(serve_fut, handle);

    serve_res.unwrap();
    handle_res.unwrap();
}
