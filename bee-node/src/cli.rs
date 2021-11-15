// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::tools::Tool;

use bee_common::logger::{LoggerConfigBuilder, LOGGER_STDOUT_NAME};
use bee_storage::backend::StorageBackend;

use log::LevelFilter;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct CliArgs {
    #[structopt(short = "c", long = "config", help = "Path of the configuration file")]
    config: Option<String>,
    #[structopt(
        short = "l",
        long = "log-level",
        help = "Stdout log level amongst \"trace\", \"debug\", \"info\", \"warn\" and \"error\""
    )]
    log_level: Option<LevelFilter>,
    #[structopt(subcommand)]
    tool: Option<Tool>,
    #[structopt(short = "v", long = "version", help = "Prints bee version")]
    version: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl CliArgs {
    pub fn new() -> Self {
        Self::from_args()
    }

    pub fn config(&self) -> Option<&String> {
        self.config.as_ref()
    }

    pub fn log_level(&self) -> Option<&LevelFilter> {
        self.log_level.as_ref()
    }

    pub fn tool(&self) -> Option<&Tool> {
        self.tool.as_ref()
    }

    pub fn version(&self) -> bool {
        self.version
    }
}
