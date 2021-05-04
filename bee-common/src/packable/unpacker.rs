// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub trait Unpacker {
    type Error: UnpackError;

    fn unpack_exact_bytes<const N: usize>(&mut self) -> Result<&[u8; N], Self::Error>;

    fn unpack_bytes(&mut self, n: usize) -> Result<&[u8], Self::Error>;
}

pub trait UnpackError {
    fn invalid_variant(value: u64) -> Self;
}
