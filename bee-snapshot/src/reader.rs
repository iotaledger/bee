// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;

use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
};

pub fn snapshot_reader(path: &Path) -> Result<BufReader<File>, Error> {
    Ok(BufReader::new(
        OpenOptions::new().read(true).open(path).map_err(Error::Io)?,
    ))
}
