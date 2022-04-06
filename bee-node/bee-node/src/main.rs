// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{error::Error, path::Path};

use bee_identity::Identity;
#[cfg(feature = "trace")]
use bee_node::trace;
use bee_node::{
    print_banner_and_version, tools, ClArgs, EntryNodeBuilder, EntryNodeConfig, FullNodeBuilder, FullNodeConfig,
    NodeConfig, NodeConfigBuilder,
};
use bee_plugin_mps::MpsPlugin;
use bee_runtime::node::NodeBuilder as _;
#[cfg(feature = "rocksdb")]
use bee_storage_rocksdb::storage::Storage;
#[cfg(all(feature = "sled", not(feature = "rocksdb")))]
use bee_storage_sled::storage::Storage;

const CONFIG_PATH_DEFAULT: &str = "./config.json";
const IDENTITY_PATH_DEFAULT: &str = "./identity.key";

/// The entry point of the node software.
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let pid = std::process::id();

    // Load the command line arguments passed to the binary.
    let cl_args = ClArgs::load();

    // Deserialize the config.
    let identity_path = cl_args
        .identity_path()
        .unwrap_or_else(|| Path::new(IDENTITY_PATH_DEFAULT))
        .to_owned();
    let (identity_field, config) = deserialize_config(&cl_args, pid);

    // Initialize the logger.
    #[cfg(not(feature = "trace"))]
    fern_logger::logger_init(config.logger().clone())?;

    // Initialize the subscriber.
    #[cfg(feature = "trace")]
    let flamegrapher = trace::init(config.logger().clone(), config.tracing().clone())?;

    // Restore or create new identity.
    let identity = Identity::restore_or_new(identity_path, identity_field)?;

    // Execute one of Bee's tools and exit.
    if let Some(tool) = cl_args.tool() {
        return tools::exec(tool, &identity, &config).map_err(|e| e.into());
    }

    // Just show the version and exit.
    if cl_args.print_commit_version() {
        print_banner_and_version(false);
        return Ok(());
    }

    print_banner_and_version(true);

    // Start running the node.
    if config.run_as_entry_node() {
        start_entrynode(identity, config).await;
    } else {
        start_fullnode(identity, config).await;
    }

    #[cfg(feature = "trace")]
    if let Some(f) = flamegrapher {
        f.write_flamegraph()?;
    }

    Ok(())
}

fn deserialize_config(cl_args: &ClArgs, pid: u32) -> (Option<String>, NodeConfig<Storage>) {
    match NodeConfigBuilder::<Storage>::from_file(
        cl_args.config_path().unwrap_or_else(|| Path::new(CONFIG_PATH_DEFAULT)),
    ) {
        Ok(builder) => builder.apply_args(cl_args).finish(pid),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    }
}

#[cfg_attr(feature = "trace", trace_tools::observe)]
async fn start_entrynode(identity: Identity, config: NodeConfig<Storage>) {
    let entry_node_config = EntryNodeConfig::from(identity, config);
    let node_builder = EntryNodeBuilder::new(entry_node_config);

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

#[cfg_attr(feature = "trace", trace_tools::observe)]
async fn start_fullnode(identity: Identity, config: NodeConfig<Storage>) {
    let full_node_config = FullNodeConfig::from(identity, config);
    let node_builder = FullNodeBuilder::<Storage>::new(full_node_config);

    match node_builder {
        Ok(builder) => match builder.with_plugin::<MpsPlugin>().finish().await {
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
