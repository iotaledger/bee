// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use bee_common::packable::Packable;
pub use bee_message::milestone::MilestoneIndex;

#[test]
fn debug_impl() {
    assert_eq!(format!("{:?}", MilestoneIndex::new(0)), "MilestoneIndex(0)",);
}

#[test]
fn display_impl() {
    assert_eq!(format!("{}", MilestoneIndex::new(0)), "0");
}

#[test]
fn unpack() {
    let packed = 0u32.pack_new();
    assert_eq!(
        MilestoneIndex::unpack(&mut packed.as_slice()).unwrap(),
        MilestoneIndex(0)
    );
}

#[test]
fn add_u32() {
    let sum = MilestoneIndex(1) + 2;
    assert_eq!(sum, MilestoneIndex(3));
}

#[test]
fn add_other() {
    let sum = MilestoneIndex(1) + MilestoneIndex(2);
    assert_eq!(sum, MilestoneIndex(3));
}

#[test]
fn sub_u32() {
    let sub = MilestoneIndex(3) - 2;
    assert_eq!(sub, MilestoneIndex(1));
}

#[test]
fn sub_other() {
    let sub = MilestoneIndex(3) - MilestoneIndex(2);
    assert_eq!(sub, MilestoneIndex(1));
}
