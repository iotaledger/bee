// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// use bee_common::logger::logger_init;
use bee_node::{plugins, print_banner_and_version, tools, CliArgs, NodeBuilder, NodeConfigBuilder};
use bee_runtime::node::NodeBuilder as _;
use bee_storage_rocksdb::storage::Storage as Rocksdb;

use log::error;
use tracing_subscriber::prelude::*;

const CONFIG_PATH: &str = "./config.toml";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = CliArgs::new();

    let config = match NodeConfigBuilder::from_file(cli.config().unwrap_or(&CONFIG_PATH.to_owned())) {
        Ok(builder) => builder.with_cli_args(cli.clone()).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };

    // if let Err(e) = logger_init(config.logger.clone()) {
    //     panic!("Failed to initialise the logger: {}", e);
    // }

    if let Some(tool) = cli.tool() {
        if let Err(e) = tools::exec(tool) {
            error!("Tool execution failed: {}", e);
        }
        return Ok(());
    }

    print_banner_and_version();

    if cli.version() {
        return Ok(());
    }

    let (layer, server) = console_subscriber::TasksLayer::new();
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into())
        .add_directive("tokio=info".parse()?);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(layer)
        .init();

    let serve = tokio::spawn(async move { server.serve().await.unwrap() });

    match NodeBuilder::<Rocksdb>::new(config) {
        Ok(builder) => match builder.with_plugin::<plugins::Mps>().finish().await {
            Ok(node) => {
                tokio::join!(serve, node.run());
            }
            Err(e) => error!("Failed to build node: {}", e),
        },
        Err(e) => error!("Failed to build node builder: {}", e),
    }

    Ok(())
}
