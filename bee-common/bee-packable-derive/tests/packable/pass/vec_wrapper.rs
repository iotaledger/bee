// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]

use bee_packable::{packer::VecPacker, Packable, VecPrefix, error::PrefixError};

use core::convert::{TryFrom, Infallible};

#[derive(Packable)]
#[packable(pack_error = PrefixError<Infallible, <u16 as TryFrom<usize>>::Error>)]
#[packable(unpack_error = PrefixError<Infallible, <usize as TryFrom<u16>>::Error>)]
pub struct Foo {
    #[packable(wrapper = VecPrefix<u8, u16>)]
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
