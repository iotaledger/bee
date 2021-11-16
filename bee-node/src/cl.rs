// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::tools::Tool;

use log::LevelFilter;
use structopt::StructOpt;

use std::path::{Path, PathBuf};

/// A type that holds the data passed to the binary via the command line.
#[derive(Clone, Debug, StructOpt)]
pub struct ClArgs {
    // The config file location.
    #[structopt(
        short = "c",
        long = "config",
        help = "Sets a custom path to a configuration file to be used"
    )]
    config_path: Option<PathBuf>,
    // The log level to be used during runtime.
    #[structopt(
        short = "l",
        long = "log-level",
        help = "Sets the stdout log level to either \"trace\", \"debug\", \"info\", \"warn\" or \"error\""
    )]
    log_level: Option<LevelFilter>,
    // Whether a tool subcommand should be executed.
    #[structopt(subcommand)]
    tool: Option<Tool>,
    // Whether the exact commit version should be printed.
    #[structopt(short = "v", long = "commit_version", help = "Prints exact commit version")]
    commit_version: bool,
}

impl Default for ClArgs {
    fn default() -> Self {
        Self::load()
    }
}

impl ClArgs {
    /// Creates an instance of this type from the arguments passed to the binary via the command line.
    pub fn load() -> Self {
        let args = <Self as StructOpt>::from_args();
        validate_args(&args);
        args
    }

    /// Returns the config file path.
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_ref().map(|path| path.as_path())
    }

    /// Returns the chosen log level.
    pub fn log_level(&self) -> Option<LevelFilter> {
        self.log_level
    }

    /// Returns the chosen tool (subcommand).
    pub fn tool(&self) -> Option<&Tool> {
        self.tool.as_ref()
    }

    /// Returns whether the exact commit version should be printed.
    pub fn commit_version(&self) -> bool {
        self.commit_version
    }
}

fn validate_args(args: &ClArgs) -> bool {
    if let Some(file_path) = args.config_path() {
        if !file_path.exists() {
            return false;
        }
    }
    true
}
