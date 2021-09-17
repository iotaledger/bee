// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![recursion_limit = "256"]

use bee_node::{plugins, print_banner_and_version, tools, CliArgs, NodeBuilder, NodeConfigBuilder};
use bee_runtime::{
    node::NodeBuilder as _,
    task::{StandaloneSpawner, TaskSpawner},
};
#[cfg(feature = "rocksdb")]
use bee_storage_rocksdb::storage::Storage as RocksDb;
#[cfg(all(feature = "sled", not(feature = "rocksdb")))]
use bee_storage_sled::storage::Storage as Sled;

use log::error;

const CONFIG_PATH: &str = "./config.toml";

#[cfg(feature = "console")]
fn logger_init() -> tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    use tracing_subscriber::prelude::*;

    let (layer, server) = console_subscriber::TasksLayer::new();

    // Unwrap here is fine, since it is known that the string is correct.
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into())
        .add_directive("tokio=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(layer)
        .init();

    StandaloneSpawner::spawn(async move { server.serve().await })
}

#[cfg(not(feature = "console"))]
fn logger_init(logger: bee_common::logger::LoggerConfig) {
    use bee_common::logger;

    if let Err(e) = logger::logger_init(logger) {
        panic!("Failed to initialise the logger: {}", e);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = CliArgs::new();

    if let Some(tool) = cli.tool() {
        return tools::exec(tool).map_err(|e| e.into());
    }

    print_banner_and_version();

    if cli.version() {
        return Ok(());
    }

    let config = match NodeConfigBuilder::from_file(cli.config().unwrap_or(&CONFIG_PATH.to_owned())) {
        Ok(builder) => builder.with_cli_args(cli.clone()).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };

    #[cfg(feature = "console")]
    let console_handle = logger_init();

    #[cfg(not(feature = "console"))]
    logger_init(config.logger.clone());

    #[cfg(feature = "rocksdb")]
    let node_builder = NodeBuilder::<RocksDb>::new(config);
    #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
    let node_builder = NodeBuilder::<Sled>::new(config);

    match node_builder {
        Ok(builder) => match builder.with_plugin::<plugins::Mps>().finish().await {
            Ok(node) => {
                #[cfg(feature = "console")]
                let res = tokio::try_join!(StandaloneSpawner::spawn(node.run()), console_handle);

                #[cfg(not(feature = "console"))]
                let res = node.run().await;

                if let Err(e) = res {
                    error!("Failed to run node: {}", e);
                }
            }
            Err(e) => error!("Failed to build node: {}", e),
        },
        Err(e) => error!("Failed to build node builder: {}", e),
    }

    Ok(())
}
