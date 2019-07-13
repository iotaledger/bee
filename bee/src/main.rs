//! Entry point for the Bee node software.

#![deny(
    bad_style,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

mod constants;

use crate::constants::CUSTOM_ENV_VAR;

use bee_cli::Cli;
use bee_core::errors::Result;
use bee_core::Bee;
use bee_display::Display;

use std::env;

fn main() -> Result<()> {
    let cli = Cli::new();

    env::set_var(CUSTOM_ENV_VAR, cli.debug_level());
    pretty_env_logger::init_custom_env(CUSTOM_ENV_VAR);

    let mut display = Display::new();
    display.clear();
    display.header();

    let mut bee = Bee::new()?;

    // Start basic tasks
    bee.init();

    // Let the display join some environments of interest
    bee.join(&vec!["IP_ADDR", "CPU_LOAD", "MEM_USE", "TPS"], display.clone());

    // Additionally update the display regularly
    bee.spawn(display);

    // Blocking call until termination signal observed
    bee.run()?;

    env::remove_var(CUSTOM_ENV_VAR);

    Ok(())
}
