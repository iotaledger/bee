// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::Packable;

// repr type is used as tag type and discriminant values are used as tags.
#[derive(Packable)]
#[repr(u8)]
pub enum A {
    B = 0,
    C = 1,
}

// same as the previous struct but one of the variants uses a tag attribute instead of the
// discriminant.
#[derive(Packable)]
#[repr(u8)]
pub enum D {
    E = 0,
    #[packable(tag = 1)]
    F,
}

// repr type is used as tag type but tags are specified with attributes, discriminants should be
// ignored while packing.
#[derive(Packable)]
#[repr(u8)]
pub enum G {
    #[packable(tag = 0)]
    H = 1,
    #[packable(tag = 1)]
    I = 0,
}

// tag type is specified with an attribute, repr type should be ignored while packing.
#[derive(Packable)]
#[repr(u8)]
#[packable(tag_type = u16)]
pub enum J {
    K = 0,
    L = 1,
}

fn main() {
    assert_eq!(A::B.pack_to_vec(), 0u8.pack_to_vec());
    assert_eq!(A::C.pack_to_vec(), 1u8.pack_to_vec());

    assert_eq!(D::E.pack_to_vec(), 0u8.pack_to_vec());
    assert_eq!(D::F.pack_to_vec(), 1u8.pack_to_vec());

    assert_eq!(G::H.pack_to_vec(), 0u8.pack_to_vec());
    assert_eq!(G::I.pack_to_vec(), 1u8.pack_to_vec());

    assert_eq!(J::K.pack_to_vec(), 0u16.pack_to_vec());
    assert_eq!(J::L.pack_to_vec(), 1u16.pack_to_vec());
}
