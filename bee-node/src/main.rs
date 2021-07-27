// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;
use bee_logger::logger_init;
use bee_node::{
    banner::print_logo_and_version,
    cli::NodeCliArgs,
    config::{NodeConfigBuilder, DEFAULT_NODE_CONFIG_PATH},
};
use bee_plugin::{
    event::{DummyEvent, SillyEvent},
    hotloading::Hotloader,
    UniqueId,
};
use tokio::time::{sleep, Duration};

use std::sync::Arc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = NodeCliArgs::new();

    let config = match NodeConfigBuilder::from_file(cli.config().unwrap_or(DEFAULT_NODE_CONFIG_PATH)) {
        Ok(builder) => builder.with_cli_args(cli.clone()).finish(),
        Err(e) => panic!("failed to create the node config builder: {}", e),
    };

    if let Err(e) = logger_init(config.logger) {
        panic!("failed to initialise the logger: {}", e);
    }

    print_logo_and_version();

    if cli.version() {
        return Ok(());
    }

    let event_bus = Arc::new(EventBus::<UniqueId>::new());

    let hotloader = Hotloader::new("./plugins", Arc::clone(&event_bus));

    let handle = tokio::spawn(async move { hotloader.run().await });

    {
        let event_bus = Arc::clone(&event_bus);
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                event_bus.dispatch(DummyEvent {})
            }
        });
    }

    tokio::spawn(async move {
        for _ in 0..1000 {
            sleep(Duration::from_millis(1)).await;
            event_bus.dispatch(SillyEvent {})
        }
    });

    handle.await??;

    Ok(())
}
