// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    packer::VecPacker,
    Packable, VecPrefix,
};

use core::convert::Infallible;

#[derive(Packable)]
#[packable(pack_error = PackPrefixError<Infallible, u16>)]
#[packable(unpack_error = UnpackPrefixError<Infallible, u16>)]
pub struct Foo {
    #[packable(wrapper = VecPrefix<u8, u16, 2>)]
    inner: Vec<u8>,
}

fn main() {
    let mut packer = VecPacker::new();
    let value = Foo { inner: vec![13, 37] };
    assert_eq!(
        value.packed_len(),
        core::mem::size_of::<u16>() + 2 * core::mem::size_of::<u8>()
    );
    value.pack(&mut packer).unwrap();
    assert_eq!(value.packed_len(), packer.len());
}
