// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{error::Error, path::Path};

use bee_identity::{pem_file, Identity, Keypair};
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
use log::{error, info, warn};

const KEYPAIR_STR_LENGTH: usize = 128;

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

    // Establish identity.
    let keypair = match pem_file::read_keypair_from_pem_file(&identity_path) {
        Ok(keypair) => {
            if identity_field.is_some() {
                warn!(
                    "The config file contains an `identity` field which will be ignored. You may safely delete this field to suppress this warning."
                );
            }
            Ok(keypair)
        }
        Err(pem_file::PemFileError::Read(_)) => {
            // If we can't read from the file (which means it probably doesn't exist) we either migrate from the
            // existing config or generate a new identity.
            let keypair = if let Some(identity_encoded) = identity_field {
                warn!(
                    "There is no identity file at `{}`. Migrating identity from the existing config file.",
                    identity_path.display(),
                );

                migrate_keypair(identity_encoded).map_err(|e| {
                    error!("Failed to migrate keypair: {}", e);
                    e
                })?
            } else {
                info!(
                    "There is no identity file at `{}`. Generating a new one.",
                    identity_path.display()
                );

                Keypair::generate()
            };

            pem_file::write_keypair_to_pem_file(identity_path, &keypair).map_err(|e| {
                error!("Failed to write PEM file: {}", e);
                e
            })?;

            Ok(keypair)
        }
        Err(e) => {
            error!("Could not extract private key from PEM file: {}", e);
            Err(e)
        }
    }?;

    let identity = Identity::from_keypair(keypair);

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

#[derive(Debug, thiserror::Error)]
enum IdentityMigrationError {
    #[error("hex decoding failed")]
    DecodeHex,
    #[error("keypair decoding failed")]
    DecodeKeypair,
    #[error("invalid keypair")]
    InvalidKeypair,
}

fn migrate_keypair(encoded: String) -> Result<Keypair, IdentityMigrationError> {
    if encoded.len() == KEYPAIR_STR_LENGTH {
        // Decode the keypair from hex.
        let mut decoded = [0u8; 64];
        hex::decode_to_slice(&encoded[..], &mut decoded).map_err(|_| IdentityMigrationError::DecodeHex)?;

        // Decode the keypair from bytes.
        Keypair::decode(&mut decoded).map_err(|_| IdentityMigrationError::DecodeKeypair)
    } else {
        Err(IdentityMigrationError::InvalidKeypair)
    }
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
