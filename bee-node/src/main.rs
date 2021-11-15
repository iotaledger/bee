// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![recursion_limit = "256"]

use bee_node::{
    plugins, print_banner_and_version, tools, CliArgs, EntryNodeBuilder, EntryNodeConfig, FullNodeBuilder,
    NodeConfigBuilder,
};

use bee_runtime::node::NodeBuilder as _;
#[cfg(feature = "rocksdb")]
use bee_storage_rocksdb::storage::Storage as RocksDb;
#[cfg(all(feature = "sled", not(feature = "rocksdb")))]
use bee_storage_sled::storage::Storage as Sled;

use log::error;

const CONFIG_PATH: &str = "./config.toml";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CliArgs::new();

    // Execute one of Bee's tools and exit.
    if let Some(tool) = cli.tool() {
        return tools::exec(tool).map_err(|e| e.into());
    }

    // Just show the version and exit.
    if cli.version() {
        print_banner_and_version(false);
        return Ok(());
    }

    print_banner_and_version(true);

    // Deserialize the config.
    #[cfg(feature = "rocksdb")]
    let config = match NodeConfigBuilder::<RocksDb>::from_file(cli.config().unwrap_or(&CONFIG_PATH.to_owned())) {
        Ok(builder) => builder.with_cli_args(cli.clone()).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };
    #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
    let config = match NodeConfigBuilder::<Sled>::from_file(cli.config().unwrap_or(&CONFIG_PATH.to_owned())) {
        Ok(builder) => builder.with_cli_args(cli.clone()).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };

    // Initialize the logger.
    bee_common::logger::logger_init(config.logger.clone())?;

    if config.run_as_entry_node() {
        let entry_node_cfg: EntryNodeConfig = config.into();

        // Run an autopeering entry node.
        let node = EntryNodeBuilder::new(entry_node_cfg)
            .expect("error building entry node")
            .finish()
            .await
            .expect("error initializing entry node");

        node.run().await?;
    } else {
        // Run a full node with RocksDB storage backend.
        #[cfg(feature = "rocksdb")]
        let node_builder = FullNodeBuilder::<RocksDb>::new(config.into());
        // Run a full node with Sled storage backend.
        #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
        let node_builder = FullNodeBuilder::<Sled>::new(config.into());

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
    }

    Ok(())
}
