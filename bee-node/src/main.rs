// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::Keypair;
use bee_node::{
    plugins, print_banner_and_version, read_keypair_from_pem_file, tools, write_keypair_to_pem_file, ClArgs,
    EntryNodeBuilder, EntryNodeConfig, FullNodeBuilder, FullNodeConfig, Local, NodeConfig, NodeConfigBuilder,
    PemFileError,
};
use bee_runtime::node::NodeBuilder as _;

#[cfg(feature = "rocksdb")]
use bee_storage_rocksdb::storage::Storage;
#[cfg(all(feature = "sled", not(feature = "rocksdb")))]
use bee_storage_sled::storage::Storage;

use log::{info, warn};

use std::{error::Error, path::Path};

const CONFIG_PATH: &str = "./config.toml";
const IDENTITY_PATH: &str = "./identity.pem";

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
    let identity_path = cl_args.identity_path().unwrap_or(Path::new(IDENTITY_PATH)).to_owned();
    let (alias, has_identity_field, config) = deserialize_config(cl_args);

    // Initialize the logger.
    let logger_cfg = config.logger_config().clone();
    fern_logger::logger_init(logger_cfg)?;

    // Establish identity.

    // The identity used to be stored in the config file.
    if has_identity_field {
        warn!("The config file contains an identity field which will be ignored.");
    }

    let keypair = match read_keypair_from_pem_file(&identity_path) {
        Ok(keypair) => keypair,
        Err(PemFileError::FileRead(_)) => {
            info!(
                "There is no identity file at `{}`. Generating a new one.",
                identity_path.display()
            );
            let keypair = Keypair::generate();
            if let Err(e) = write_keypair_to_pem_file(identity_path, &keypair) {
                panic!("Failed to write PEM file: {}", e);
            };
            keypair
        }
        Err(e) => panic!("Failed to establish identity: {}", e),
    };

    let local = Local::from_keypair(keypair, alias);

    // Start running the node.
    if config.run_as_entry_node() {
        start_entrynode(local, config).await;
    } else {
        start_fullnode(local, config).await;
    }

    Ok(())
}

fn deserialize_config(cl_args: ClArgs) -> (Option<String>, bool, NodeConfig<Storage>) {
    match NodeConfigBuilder::<Storage>::from_file(cl_args.config_path().unwrap_or(Path::new(CONFIG_PATH))) {
        Ok(builder) => builder.apply_args(&cl_args).finish(),
        Err(e) => panic!("Failed to create the node config builder: {}", e),
    }
}

async fn start_entrynode(local: Local, config: NodeConfig<Storage>) {
    let entry_node_config = EntryNodeConfig::from(local, config);
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

async fn start_fullnode(local: Local, config: NodeConfig<Storage>) {
    let full_node_config = FullNodeConfig::from(local, config);
    let node_builder = FullNodeBuilder::<Storage>::new(full_node_config);

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
