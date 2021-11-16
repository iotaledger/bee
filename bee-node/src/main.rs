// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![recursion_limit = "256"]

use bee_node::{
    plugins, print_banner_and_version, tools, ClArgs, EntryNodeBuilder, FullNodeBuilder, NodeConfigBuilder,
};

use bee_runtime::node::NodeBuilder as _;
#[cfg(feature = "rocksdb")]
use bee_storage_rocksdb::storage::Storage as RocksDb;
#[cfg(all(feature = "sled", not(feature = "rocksdb")))]
use bee_storage_sled::storage::Storage as Sled;

use std::{error::Error, path::Path};

const CONFIG_PATH: &str = "./config.toml";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load the command line arguments passed to the binary.
    let cl_args = ClArgs::load();

    // Execute one of Bee's tools and exit.
    if let Some(tool) = cl_args.tool() {
        return tools::exec(tool).map_err(|e| e.into());
    }

    // Just show the version and exit.
    if cl_args.commit_version() {
        print_banner_and_version(false);
        return Ok(());
    }

    print_banner_and_version(true);

    // Deserialize the config.
    #[cfg(feature = "rocksdb")]
    let config = match NodeConfigBuilder::<RocksDb>::from_file(cl_args.config_path().unwrap_or(Path::new(CONFIG_PATH)))
    {
        Ok(builder) => builder.with_cli_args(cl_args.clone()).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };
    #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
    let config = match NodeConfigBuilder::<Sled>::from_file(cli.config_path().unwrap_or(Path::new(CONFIG_PATH))) {
        Ok(builder) => builder.with_cli_args(cli.clone()).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };

    // Initialize the logger.
    bee_common::logger::logger_init(config.logger().clone())?;

    if config.run_as_entry_node() {
        // Run as an autopeering entry node.
        let node = EntryNodeBuilder::new(config.into())
            .expect("error building entry node")
            .finish()
            .await
            .expect("error initializing entry node");

        node.run().await?;
    } else {
        // Run as a full node.
        #[cfg(feature = "rocksdb")]
        let node_builder = FullNodeBuilder::<RocksDb>::new(config.into());
        #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
        let node_builder = FullNodeBuilder::<Sled>::new(config.into());

        match node_builder {
            Ok(builder) => match builder.with_plugin::<plugins::Mps>().finish().await {
                Ok(node) => {
                    let res = node.run().await;

                    if let Err(e) = res {
                        log::error!("Failed to run node: {}", e);
                    }
                }
                Err(e) => log::error!("Failed to build node: {}", e),
            },
            Err(e) => log::error!("Failed to build node builder: {}", e),
        }
    }

    Ok(())
}
