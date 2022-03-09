// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

///
pub trait StateTransition {
    ///
    fn creation(next_state: &Self);

    ///
    fn transition(current_state: &Self, next_state: &Self);

    ///
    fn destruction(current_state: &Self);
}
