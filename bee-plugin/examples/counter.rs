// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_plugin::{
    event::{DummyEvent, EventId},
    plugin::{serve_plugin, Plugin},
    PluginError,
};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct Counter {
    inner: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl Plugin for Counter {
    fn handshake() -> Vec<EventId> {
        vec![EventId::Dummy]
    }

    async fn shutdown(&self) {
        println!(
            "counter was shutdown with a count of {}",
            self.inner.load(Ordering::Relaxed)
        );
    }

    async fn process_dummy_event(&self, _event: DummyEvent) {
        let count = self.inner.fetch_add(1, Ordering::Relaxed) + 1;

        if count % 100 == 0 {
            println!("the count is {}", count);
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), PluginError> {
    let counter = Counter {
        inner: Arc::new(0.into()),
    };

    serve_plugin(counter).await?;

    Ok(())
}
