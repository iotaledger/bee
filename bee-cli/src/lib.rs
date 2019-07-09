//! CLI for Bee.

#![deny(
    bad_style,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

use structopt::StructOpt;

/// CLI arguments for Bee.
#[derive(StructOpt)]
pub struct Args {
    //
    #[structopt(long, name = "debug_level", default_value = "debug")]
    debug_level: String,

    #[structopt(short, long, name = "host", default_value = "localhost")]
    host: String,

    #[structopt(short, long, name = "port", default_value = "1337")]
    port: u16,

    #[structopt(long, name = "cache_size", default_value = "50000")]
    cache_size: usize,
}

/// CLI for Bee.
pub struct Cli {
    args: Args,
}

impl Cli {
    /// Create a new command line interface.
    pub fn new() -> Self {
        Self  {
            args: Args::from_args(),
        }
    }

    /// Returns the debug level.
    pub fn debug_level(&self) -> &str {
        &self.args.debug_level
    }

    /// Returns the debug level.
    pub fn host(&self) -> &str {
        &self.args.host
    }

    /// Returns the debug level.
    pub fn port(&self) -> u16 {
        self.args.port
    }
    /// Returns the transaction cache size.
    pub fn cache_size(&self) -> usize {
        self.args.cache_size
    }
}
