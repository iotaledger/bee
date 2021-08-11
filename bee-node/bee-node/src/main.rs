// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_logger::logger_init;
use bee_node::{
    banner::print_logo_and_version,
    cli::NodeCliArgs,
    config::{NodeConfigBuilder, DEFAULT_NODE_CONFIG_PATH},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = NodeCliArgs::new();

    let config = match NodeConfigBuilder::from_file(cli.config().unwrap_or(DEFAULT_NODE_CONFIG_PATH)) {
        Ok(builder) => builder.with_cli_args(cli.clone()).finish(),
        Err(e) => panic!("failed to create the node config builder: {}", e),
    };

    if let Err(e) = logger_init(config.logger) {
        panic!("failed to initialise the logger: {}", e);
    }

    print_logo_and_version();

    if cli.version() {
        return Ok(());
    }

    Ok(())
}
