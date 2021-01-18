// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::config::ExampleConfig;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pingpong", about = "bee-network example")]
pub struct Args {
    #[structopt(short = "b", long = "bind")]
    bind_address: String,

    #[structopt(short = "p", long = "peers")]
    peer_addresses: Vec<String>,
}

impl Args {
    pub fn into_config(self) -> ExampleConfig {
        let Args {
            bind_address,
            mut peer_addresses,
        } = self;

        let mut config = ExampleConfig::build().with_bind_address(bind_address);

        for peer_address in peer_addresses.drain(..) {
            config = config.with_peer_address(peer_address);
        }

        config.finish()
    }
}
