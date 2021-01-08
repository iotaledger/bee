// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pingpong", about = "bee-network example")]
pub struct Args {
    #[structopt(short = "b", long = "bind")]
    pub bind_address: String,

    #[structopt(short = "p", long = "peers")]
    pub peer_addresses: Vec<String>,

    #[structopt(short = "m", long = "msg")]
    pub message: String,
}
