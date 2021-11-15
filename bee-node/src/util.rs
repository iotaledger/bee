// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{BEE_GIT_COMMIT, BEE_NAME, BEE_VERSION};

use bee_runtime::node::NodeInfo;

use crypto::hashes::{blake2b::Blake2b256, Digest};

use std::convert::TryInto;

/// Creates the corresponding network id from a network name.
pub(crate) fn create_id_from_network_name(network_name: &impl AsRef<str>) -> u64 {
    // Panic: unwrapping is fine because of the guarantee of the hash length and the correct bounds.
    u64::from_le_bytes(
        Blake2b256::digest(network_name.as_ref().as_bytes())[0..8]
            .try_into()
            .unwrap(),
    )
}

/// Determines name and current version of the node.
pub(crate) fn create_node_info() -> NodeInfo {
    NodeInfo {
        name: BEE_NAME.to_owned(),
        version: if BEE_GIT_COMMIT.is_empty() {
            BEE_VERSION.to_owned()
        } else {
            BEE_VERSION.to_owned() + "-" + &BEE_GIT_COMMIT[0..7]
        },
    }
}

/// Prints the Bee logo to the terminal.
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
