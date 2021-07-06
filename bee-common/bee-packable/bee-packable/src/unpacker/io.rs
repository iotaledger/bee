// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate std;

use crate::unpacker::Unpacker;

use std::io::{self, Read};

impl<R: Read> Unpacker for R {
    type Error = io::Error;

    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, mut bytes: B) -> Result<(), Self::Error> {
        self.read_exact(bytes.as_mut())
    }
}
