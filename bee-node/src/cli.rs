// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{config::NodeConfigBuilder, tools::Tool};

use bee_common::logger::{LoggerConfigBuilder, LOGGER_STDOUT_NAME};
use bee_storage::backend::StorageBackend;

use log::LevelFilter;
use structopt::StructOpt;

/// A type describing all possible command-line arguments.
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
    /// Create a new instance of `CliArgs` based on the arguments that were passed to the current program.
    pub fn new() -> Self {
        Self::from_args()
    }

    /// Get the config file specified by these command-line arguments.
    pub fn config(&self) -> Option<&str> {
        self.config.as_ref().map(String::as_str)
    }

    /// Get the log level specified by these command-line arguments.
    pub fn log_level(&self) -> Option<&LevelFilter> {
        self.log_level.as_ref()
    }

    /// Get the node tool requested by these command-line arguments.
    pub fn tool(&self) -> Option<&Tool> {
        self.tool.as_ref()
    }

    /// Determine whether these command-line arguments are requesting the version to be shown.
    pub fn version(&self) -> bool {
        self.version
    }
}

impl<B: StorageBackend> NodeConfigBuilder<B> {
    /// Apply the given command-line arguments to this `NodeConfigBuilder`.
    pub fn with_cli_args(mut self, args: CliArgs) -> Self {
        if let Some(log_level) = args.log_level {
            if self.logger.is_none() {
                self.logger = Some(LoggerConfigBuilder::default());
            }
            self.logger.as_mut().unwrap().level(LOGGER_STDOUT_NAME, log_level);
        }
        self
    }
}
