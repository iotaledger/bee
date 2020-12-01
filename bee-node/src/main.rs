// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::node::{Node as _, NodeBuilder as _};
use bee_node::{default_plugins, print_banner_and_version, tools, CliArgs, Node, NodeConfigBuilder};
use bee_storage_rocksdb::storage::Storage as Rocksdb;

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

    Node::<Rocksdb>::build(config)
        .with_plugin::<default_plugins::Mps>()
        .with_logging()
        .finish()
        .await
        .expect("Failed to build node")
        .run()
        .await
        .expect("Failed to run node");
}
