// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_node::{
    plugins, print_banner_and_version, tools, ClArgs, EntryNodeBuilder, FullNodeBuilder, FullNodeConfig, NodeConfig,
    NodeConfigBuilder, NodeStorageBackend,
};
use bee_runtime::node::NodeBuilder as _;

#[cfg(feature = "rocksdb")]
use bee_storage_rocksdb::storage::Storage;
#[cfg(all(feature = "sled", not(feature = "rocksdb")))]
use bee_storage_sled::storage::Storage;

use std::{error::Error, path::Path};

const CONFIG_PATH: &str = "./config.toml";

/// The entry point of the node software.
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load the command line arguments passed to the binary.
    let cl_args = ClArgs::load();

    // Execute one of Bee's tools and exit.
    if let Some(tool) = cl_args.tool() {
        return tools::exec(tool).map_err(|e| e.into());
    }

    // Just show the version and exit.
    if cl_args.print_commit_version() {
        print_banner_and_version(false);
        return Ok(());
    }

    print_banner_and_version(true);

    // Deserialize the config.
    let config = deserialize_config(cl_args);

    // Initialize the logger.
    let logger_cfg = config.logger_config().clone();
    bee_common::logger::logger_init(logger_cfg)?;

    // Start running the node.
    if config.run_as_entry_node() {
        start_entrynode(config).await;
    } else {
        start_fullnode(config).await;
    }

    Ok(())
}

fn deserialize_config(cl_args: ClArgs) -> NodeConfig<Storage> {
    #[cfg(feature = "rocksdb")]
    let config = match NodeConfigBuilder::<Storage>::from_file(
        cl_args.config_path().unwrap_or_else(|| Path::new(CONFIG_PATH)),
    ) {
        Ok(builder) => builder.apply_args(&cl_args).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };
    #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
    let config = match NodeConfigBuilder::<Storage>::from_file(cl_args.config_path().unwrap_or(Path::new(CONFIG_PATH)))
    {
        Ok(builder) => builder.apply_args(cl_args).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    };
    config
}

async fn start_entrynode<S: NodeStorageBackend>(config: NodeConfig<S>) {
    let node_builder = EntryNodeBuilder::new(config.into());

    match node_builder {
        Ok(builder) => match builder.finish().await {
            Ok(node) => {
                if let Err(e) = node.run().await {
                    log::error!("Failed to run entry node: {}", e);
                }
            }
            Err(e) => log::error!("Failed to build entry node: {}", e),
        },
        Err(e) => log::error!("Failed to build entry node builder: {}", e),
    }
}

async fn start_fullnode<S: NodeStorageBackend>(config: NodeConfig<S>)
where
    FullNodeConfig<Storage>: From<NodeConfig<S>>,
{
    #[cfg(feature = "rocksdb")]
    let node_builder = FullNodeBuilder::<Storage>::new(config.into());
    #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
    let node_builder = FullNodeBuilder::<Storage>::new(config.into());

    match node_builder {
        Ok(builder) => match builder.with_plugin::<plugins::Mps>().finish().await {
            Ok(node) => {
                if let Err(e) = node.run().await {
                    log::error!("Failed to run full node: {}", e);
                }
            }
            Err(e) => log::error!("Failed to build full node: {}", e),
        },
        Err(e) => log::error!("Failed to build full node builder: {}", e),
    }
}
