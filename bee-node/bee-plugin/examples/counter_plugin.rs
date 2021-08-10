// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_plugin::{
    event::*,
    handshake::HandshakeInfo,
    PluginError, {serve_plugin, Plugin},
};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct Counter {
    inner: Arc<AtomicUsize>,
}

impl Counter {
    fn new() -> Self {
        Self {
            inner: Arc::new(AtomicUsize::new(0)),
        }
    }
    fn increase(&self) {
        let count = self.inner.fetch_add(1, Ordering::Relaxed) + 1;

        if count % 100 == 0 {
            println!("the count is {}", count);
        }
    }
}

#[async_trait::async_trait]
impl Plugin for Counter {
    fn handshake_info() -> HandshakeInfo {
        HandshakeInfo::new("[::1]:50051".parse().unwrap(), "counter", vec![EventId::MessageParsed])
    }

    async fn shutdown(&self) {
        println!(
            "counter was shutdown with a count of {}",
            self.inner.load(Ordering::Relaxed)
        );
    }

    async fn process_message_parsed_event(&self, _event: MessageParsedEvent) {
        self.increase();
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), PluginError> {
    let counter = Counter::new();

    serve_plugin(counter).await?;

    Ok(())
}
