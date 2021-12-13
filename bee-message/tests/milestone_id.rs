// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{payload::milestone::MilestoneId, Error};

use core::str::FromStr;

const MILESTONE_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MILESTONE_ID_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const MILESTONE_ID_INVALID_LEN_TOO_SHORT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";
const MILESTONE_ID_INVALID_LEN_TOO_LONG: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64900";

#[test]
fn debug_impl() {
    let id_bytes: [u8; 32] = hex::decode(MILESTONE_ID).unwrap().try_into().unwrap();

    assert_eq!(
        format!("{:?}", MilestoneId::new(id_bytes)),
        "MilestoneId(52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649)"
    );
}

#[test]
fn as_ref() {
    let id_bytes = hex::decode(MILESTONE_ID).unwrap().try_into().unwrap();
    let milestone = MilestoneId::new(id_bytes);

    assert_eq!(milestone.as_ref(), &id_bytes);
}

#[test]
fn from_str_valid() {
    MilestoneId::from_str(MILESTONE_ID).unwrap();
}

#[test]
fn from_str_invalid_hex() {
    assert!(matches!(
        MilestoneId::from_str(MILESTONE_ID_INVALID_HEX),
        Err(Error::InvalidHexadecimalChar(hex))
            if hex == MILESTONE_ID_INVALID_HEX
    ));
}

#[test]
fn from_str_invalid_len_too_short() {
    assert!(matches!(
        MilestoneId::from_str(MILESTONE_ID_INVALID_LEN_TOO_SHORT),
        Err(Error::InvalidHexadecimalLength{expected, actual})
            if expected == MilestoneId::LENGTH * 2 && actual == MilestoneId::LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_invalid_len_too_long() {
    assert!(matches!(
        MilestoneId::from_str(MILESTONE_ID_INVALID_LEN_TOO_LONG),
        Err(Error::InvalidHexadecimalLength{expected, actual})
            if expected == MilestoneId::LENGTH * 2 && actual == MilestoneId::LENGTH * 2 + 2
    ));
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
    assert_eq!(milestone_id.pack_new().len(), 32);
}

// Validate that a `unpack` ∘ `pack` round-trip results in the original milestone id.
#[test]
fn pack_unpack_valid() {
    let milestone_id = MilestoneId::from_str(MILESTONE_ID).unwrap();
    let packed_milestone_id = milestone_id.pack_new();

    assert_eq!(packed_milestone_id.len(), milestone_id.packed_len());
    assert_eq!(
        milestone_id,
        Packable::unpack(&mut packed_milestone_id.as_slice()).unwrap()
    );
}
