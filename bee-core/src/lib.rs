//! Core functionality

#![deny(warnings, bad_style, missing_docs)]

mod constants;

use constants::{NAME, VERSION};

/// Returns a nice greeting.
pub fn get_name() -> String {
    format!("{} v{}", NAME, VERSION)
}
