// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

#[test]
fn kind() {
    assert_eq!(ReferenceUnlock::KIND, 1);
}
