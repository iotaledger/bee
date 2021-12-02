// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::{
    packer::{IoPacker, SlicePacker},
    unpacker::IoUnpacker,
    Packable, PackableExt,
};

use core::fmt::Debug;

fn generic_test_pack_to_slice_unpack_verified<P>(packable: &P)
where
    P: Packable + Eq + Debug,
    P::UnpackError: Debug,
{
    let mut vec = vec![0; packable.packed_len()];

    let mut packer = SlicePacker::new(&mut vec);
    packable.pack(&mut packer).unwrap();

    let unpacked = P::unpack_verified(&vec).unwrap();

    assert_eq!(packable, &unpacked);

    if vec.pop().is_some() {
        let mut packer = SlicePacker::new(&mut vec);
        packable.pack(&mut packer).unwrap_err();
    }
}

fn generic_test_pack_to_vec_unpack_verified<P>(packable: &P) -> (Vec<u8>, P)
where
    P: Packable + Eq + Debug,
    P::UnpackError: Debug,
{
    let vec = packable.pack_to_vec();
    let unpacked = P::unpack_verified(&vec).unwrap();

    assert_eq!(packable, &unpacked);
    assert_eq!(packable.packed_len(), vec.len());

    (vec, unpacked)
}

pub fn generic_test<P>(packable: &P) -> (Vec<u8>, P)
where
    P: Packable + Eq + Debug,
    P::UnpackError: Debug,
{
    // Tests for `Vec` and `&[u8]`

    let mut vec = Vec::new();
    packable.pack(&mut vec).unwrap();
    let unpacked = P::unpack_verified(&mut vec.as_slice()).unwrap();
    assert_eq!(packable, &unpacked);

    // Tests for `Read` and `Write`

    let mut packer = IoPacker::new(Vec::new());
    packable.pack(&mut packer).unwrap();
    let mut unpacker = IoUnpacker::new(packer.as_slice());
    let unpacked = P::unpack::<_, true>(&mut unpacker).unwrap();
    assert_eq!(packable, &unpacked);

    generic_test_pack_to_slice_unpack_verified(packable);
    generic_test_pack_to_vec_unpack_verified(packable)
}
