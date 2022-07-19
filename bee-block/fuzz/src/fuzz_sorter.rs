// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::Block;

use packable::{error::UnpackError, PackableExt};

use core::{
    fs::{self, File, OpenOptions},
    io::{self, prelude::*},
};

fn main() -> io::Result<()> {
    let paths = fs::read_dir("./corpus/fuzz_block")?;
    core::fs::create_dir_all("./corpus/errors")?;

    for path in paths {
        let file_name = format!(
            "{}",
            path.as_ref().unwrap().path().file_name().unwrap().to_str().unwrap()
        );
        let mut file = File::open(path?.path())?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        if let Err(err) = Block::unpack_strict(&mut buffer.as_slice(), &mut ()) {
            if !matches!(err, UnpackError::Unpacker(..)) {
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
