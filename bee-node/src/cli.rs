// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::tools::Tool;

use log::LevelFilter;
use structopt::StructOpt;

use std::path::{Path, PathBuf};

/// Holds the command line arguments that were passed to the binary.
pub struct ClArgs {
    cli: Cli,
}

// The command-line interface.
// BEWARE: `structopt` puts any doc comment for this struct into the output of `--help`. We don't want that.
#[derive(Clone, Debug, StructOpt)]
struct Cli {
    // The config file location.
    #[structopt(
        short = "c",
        long = "config",
        help = "Sets a custom path to a configuration file to be used"
    )]
    config_path: Option<PathBuf>,
    #[structopt(
        short = "i",
        long = "identity",
        help = "Sets a custom path to the identity file to be used"
    )]
    identity_path: Option<PathBuf>,
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
    // Whether the node should run as an (autopeering) entry node.
    #[structopt(long = "entry_node", help = "Runs as autopeering entry node")]
    entry_node: bool,
}

impl Default for ClArgs {
    fn default() -> Self {
        Self::load()
    }
}

impl ClArgs {
    /// Creates an instance of this type from the arguments passed to the binary via the command line.
    pub fn load() -> Self {
        let cli = <Cli as StructOpt>::from_args();
        let args = Self { cli };
        validate_args(&args);
        args
    }

    /// Returns the config file path.
    pub fn config_path(&self) -> Option<&Path> {
        self.cli.config_path.as_deref()
    }

    /// Returns the identity file path.
    pub fn identity_path(&self) -> Option<&Path> {
        self.cli.identity_path.as_deref()
    }

    /// Returns the chosen log level.
    pub fn log_level(&self) -> Option<LevelFilter> {
        self.cli.log_level
    }

    /// Returns the chosen tool (subcommand).
    pub fn tool(&self) -> Option<&Tool> {
        self.cli.tool.as_ref()
    }

    /// Returns whether the exact commit version should be printed.
    pub fn print_commit_version(&self) -> bool {
        self.cli.commit_version
    }

    /// Returns whether the node should run as an (autopeering) entry node.
    pub fn run_as_entry_node(&self) -> bool {
        self.cli.entry_node
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
