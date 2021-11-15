// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::constants::{BEE_GIT_COMMIT, BEE_VERSION};

pub fn print_banner_and_version(print_banner: bool) {
    let version = if BEE_GIT_COMMIT.is_empty() {
        BEE_VERSION.to_owned()
    } else {
        BEE_VERSION.to_owned() + "-" + &BEE_GIT_COMMIT[0..7]
    };
    if print_banner {
        println!(
            "
██████╗ ███████╗███████╗
██╔══██╗██╔════╝██╔════╝
██████╦╝█████╗  █████╗
██╔══██╗██╔══╝  ██╔══╝
██████╦╝███████╗███████╗
╚═════╝ ╚══════╝╚══════╝
{: ^24}\n",
            version
        );
    } else {
        println!("{}", version);
    }
}
