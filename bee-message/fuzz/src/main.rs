// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::{self, File},
    io::{self, prelude::*},
};

use bee_common::packable::Packable;
use bee_message::{Error, Message};

fn main() -> io::Result<()> {
    let paths = fs::read_dir("./corpus/fuzz_message")?;

    for path in paths {
        let mut file = File::open(path?.path())?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        let res = Message::unpack(&mut buffer.as_slice());

        if !matches!(res, Err(Error::Io(..))) {
            println!("{:?}", res);
        }
    }

    Ok(())
}
