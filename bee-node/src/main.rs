// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common_pt2::node::{Node as _, NodeBuilder as _};
use bee_node::{plugins, print_banner_and_version, tools, CliArgs, Node, NodeConfigBuilder};
use bee_storage_rocksdb::storage::Storage as Rocksdb;

use log::error;

const CONFIG_PATH: &str = "./config.toml";

#[tokio::main]
async fn main() {
    let cli = CliArgs::new();

    if let Some(tool) = cli.tool() {
        tools::exec(tool);
        return;
    }

    print_banner_and_version();

    if cli.version() {
        return;
    }

    let config = NodeConfigBuilder::from_file(match cli.config() {
        Some(path) => path,
        None => CONFIG_PATH,
    })
    .expect("Error when creating node config builder")
    .with_cli_args(cli)
    .finish();

    match Node::<Rocksdb>::build(config)
        .with_plugin::<plugins::Mps>()
        .with_logging()
        .finish()
        .await
    {
        Ok(node) => {
            if let Err(e) = node.run().await {
                error!("Failed to run node: {}", e)
            }
        }
        Err(e) => error!("Failed to build node: {}", e),
    }
}
