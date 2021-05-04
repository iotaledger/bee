// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub trait Packer {
    type Error: PackError;

    fn pack_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

pub trait PackError {}
