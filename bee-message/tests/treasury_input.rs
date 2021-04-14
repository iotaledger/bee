// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::str::FromStr;

const MILESTONE_ID_VALID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MILESTONE_ID_INVALID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";

#[test]
fn kind() {
    assert_eq!(TreasuryInput::KIND, 1);
}

#[test]
fn new_valid() {
    let milestone_id = MilestoneId::from_str(MILESTONE_ID_VALID).unwrap();
    let input = TreasuryInput::new(milestone_id);

    assert_eq!(*input.milestone_id(), milestone_id);
    assert_eq!(*input, milestone_id);
}

#[test]
fn from_valid() {
    let milestone_id = MilestoneId::from_str(MILESTONE_ID_VALID).unwrap();
    let input: TreasuryInput = milestone_id.into();

    assert_eq!(*input.milestone_id(), milestone_id);
    assert_eq!(*input, milestone_id);
}

#[test]
fn from_str_valid() {
    let milestone_id = MilestoneId::from_str(MILESTONE_ID_VALID).unwrap();
    let input = TreasuryInput::from_str(MILESTONE_ID_VALID).unwrap();

    assert_eq!(*input.milestone_id(), milestone_id);
    assert_eq!(*input, milestone_id);
}

#[test]
fn from_str_invalid() {
    assert!(matches!(
        TreasuryInput::from_str(MILESTONE_ID_INVALID),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_to_str() {
    assert_eq!(
        TreasuryInput::from_str(MILESTONE_ID_VALID).unwrap().to_string(),
        MILESTONE_ID_VALID
    );
}

#[test]
fn packed_len() {
    let treasury_input = TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID_VALID).unwrap());

    assert_eq!(treasury_input.packed_len(), 32);
    assert_eq!(treasury_input.pack_new().len(), 32);
}

#[test]
fn pack_unpack_valid() {
    let input_1 = TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID_VALID).unwrap());
    let input_2 = TreasuryInput::unpack(&mut input_1.pack_new().as_slice()).unwrap();

    assert_eq!(input_1, input_2);
}
