//! Entry point for the Bee node software.

#![deny(warnings, bad_style, unsafe_code)]

mod constants;

use crate::constants::{CUSTOM_ENV_VAR, DEBUG_LEVEL};

use bee_core::Bee;
use bee_display::Display;

use std::env;

fn main() {
    env::set_var(CUSTOM_ENV_VAR, DEBUG_LEVEL);
    pretty_env_logger::init_custom_env(CUSTOM_ENV_VAR);

    let mut display = Display::new();
    display.clear();
    display.header();
    display.heartbeat();

    let mut bee = Bee::new();
    bee.init();
    bee.run();
}
