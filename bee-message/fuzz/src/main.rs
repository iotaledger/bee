// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::{self, File, OpenOptions},
    io::{self, prelude::*},
};

use bee_common::packable::Packable;
use bee_message::{Error, Message};

fn main() -> io::Result<()> {
    let paths = fs::read_dir("./corpus/fuzz_message")?;
    std::fs::create_dir_all("./corpus/errors")?;

    for path in paths {
        let file_name = format!(
            "{}",
            path.as_ref()
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap()
        );
        let mut file = File::open(path?.path())?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        if let Err(err) = Message::unpack(&mut buffer.as_slice()) {
            if !matches!(err, Error::Io(..)) {
                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(format!("./corpus/errors/{}", file_name))?;
                file.write_all(&buffer)?;
                println!("{:?}", err);
            }
        }
    }

    Ok(())
}
