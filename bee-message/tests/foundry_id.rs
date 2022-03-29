// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::AliasAddress,
    output::{AliasId, FoundryId, SimpleTokenScheme, TokenScheme},
};

use primitive_types::U256;

use core::str::FromStr;

#[test]
fn getters() {
    let alias_address = AliasAddress::from(AliasId::from_str("0x52fdfc072182654f163f5f0f9a621d729566c74d").unwrap());
    let serial_number = 42;
    let token_scheme =
        TokenScheme::from(SimpleTokenScheme::new(U256::from(100u8), U256::from(0u8), U256::from(100u8)).unwrap());
    let foundry_id = FoundryId::build(&alias_address, serial_number, token_scheme.kind());

    assert_eq!(foundry_id.alias_address(), alias_address);
    assert_eq!(foundry_id.serial_number(), serial_number);
    assert_eq!(foundry_id.token_scheme_kind(), token_scheme.kind());
}
