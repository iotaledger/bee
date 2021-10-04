// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![recursion_limit = "256"]

use bee_node::{plugins, print_banner_and_version, tools, CliArgs, NodeBuilder, NodeConfigBuilder};
use bee_runtime::node::NodeBuilder as _;
#[cfg(feature = "rocksdb")]
use bee_storage_rocksdb::storage::Storage as RocksDb;
#[cfg(all(feature = "sled", not(feature = "rocksdb")))]
use bee_storage_sled::storage::Storage as Sled;

use log::error;

const CONFIG_PATH: &str = "./config.toml";

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

    if let Err(e) = bee_common::logger::logger_init(config.logger.clone()) {
        panic!("Failed to initialise the logger: {}", e);
    }

    #[cfg(feature = "rocksdb")]
    let node_builder = NodeBuilder::<RocksDb>::new(config);
    #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
    let node_builder = NodeBuilder::<Sled>::new(config);

    match node_builder {
        Ok(builder) => match builder.with_plugin::<plugins::Mps>().finish().await {
            Ok(node) => {
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
