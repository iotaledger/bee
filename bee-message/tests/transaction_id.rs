// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

use core::str::FromStr;

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn from_to_str() {
    assert_eq!(
        TRANSACTION_ID,
        TransactionId::from_str(TRANSACTION_ID).unwrap().to_string()
    );
}
