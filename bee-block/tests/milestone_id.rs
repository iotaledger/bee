// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use bee_block::payload::milestone::MilestoneId;
use packable::PackableExt;

const MILESTONE_ID: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn debug_impl() {
    let id_bytes: [u8; 32] = prefix_hex::decode(MILESTONE_ID).unwrap();

    assert_eq!(
        format!("{:?}", MilestoneId::new(id_bytes)),
        "MilestoneId(0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649)"
    );
}

#[test]
fn as_ref() {
    let id_bytes = prefix_hex::decode(MILESTONE_ID).unwrap();
    let milestone = MilestoneId::new(id_bytes);

    assert_eq!(milestone.as_ref(), &id_bytes);
}

#[test]
fn from_str_valid() {
    MilestoneId::from_str(MILESTONE_ID).unwrap();
}

#[test]
fn from_to_str() {
    assert_eq!(MILESTONE_ID, MilestoneId::from_str(MILESTONE_ID).unwrap().to_string());
}

// Validate that the length of a packed `MilestoneId` matches the declared `packed_len()`.
#[test]
fn packed_len() {
    let milestone_id = MilestoneId::from_str(MILESTONE_ID).unwrap();

    assert_eq!(milestone_id.packed_len(), 32);
    assert_eq!(milestone_id.pack_to_vec().len(), 32);
}

// Validate that a `unpack` ∘ `pack` round-trip results in the original milestone id.
#[test]
fn pack_unpack_valid() {
    let milestone_id = MilestoneId::from_str(MILESTONE_ID).unwrap();
    let packed_milestone_id = milestone_id.pack_to_vec();

    assert_eq!(packed_milestone_id.len(), milestone_id.packed_len());
    assert_eq!(
        milestone_id,
        PackableExt::unpack_verified(packed_milestone_id.as_slice(), &()).unwrap()
    );
}
