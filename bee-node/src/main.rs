// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;
use bee_logger::logger_init;
use bee_node::{
    banner::print_logo_and_version,
    cli::NodeCliArgs,
    config::{NodeConfigBuilder, DEFAULT_NODE_CONFIG_PATH},
};
use bee_plugin::{server::DummyEvent, PluginManager, UniqueId};

use std::{sync::Arc, time::Duration};

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

    let mut manager = PluginManager::new(Arc::clone(&event_bus));

    let process = tokio::process::Command::new("../target/debug/examples/counter");
    let plugin_id = manager.load_plugin(process).await?;

    tokio::spawn(async move {
        for _ in 0..1000 {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            event_bus.dispatch(DummyEvent {})
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    manager.unload_plugin(plugin_id).await?;

    Ok(())
}
