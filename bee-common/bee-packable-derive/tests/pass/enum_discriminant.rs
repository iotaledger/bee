// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::Packable;

#[derive(Packable)]
#[repr(u8)]
#[packable(tag_type = u8)]
pub enum A {
    B = 0,
    C = 1,
}

#[derive(Packable)]
#[repr(u8)]
#[packable(tag_type = u8)]
pub enum D {
    E = 0,
    #[packable(tag = 1)]
    F,
}

#[derive(Packable)]
#[repr(u8)]
#[packable(tag_type = u8)]
pub enum G {
    #[packable(tag = 0)]
    H = 1,
    #[packable(tag = 1)]
    I = 0,
}

fn main() {
    assert_eq!(A::B.pack_to_vec(), 0u8.pack_to_vec());
    assert_eq!(A::C.pack_to_vec(), 1u8.pack_to_vec());

    assert_eq!(D::E.pack_to_vec(), 0u8.pack_to_vec());
    assert_eq!(D::F.pack_to_vec(), 1u8.pack_to_vec());

    assert_eq!(G::H.pack_to_vec(), 0u8.pack_to_vec());
    assert_eq!(G::I.pack_to_vec(), 1u8.pack_to_vec());
}
