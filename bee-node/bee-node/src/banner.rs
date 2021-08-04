// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the node banner to be printed.

use crate::constants::{BEE_GIT_COMMIT, BEE_NODE_VERSION};

/// Prints a banner consisting of a logo and a version.
pub fn print_logo_and_version() {
    let logo = "
██████╗ ███████╗███████╗
██╔══██╗██╔════╝██╔════╝
██████╦╝█████╗  █████╗
██╔══██╗██╔══╝  ██╔══╝
██████╦╝███████╗███████╗
╚═════╝ ╚══════╝╚══════╝
";
    let version = if BEE_GIT_COMMIT.is_empty() {
        BEE_NODE_VERSION.to_owned()
    } else {
        BEE_NODE_VERSION.to_owned() + "-" + &BEE_GIT_COMMIT[0..7]
    };
    println!("{}{: ^24}\n", logo, version);
}
