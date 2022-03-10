// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::semantic::ValidationContext;

///
pub trait StateTransition {
    ///
    fn creation(next_state: &Self, context: &ValidationContext);

    ///
    fn transition(current_state: &Self, next_state: &Self, context: &ValidationContext);

    ///
    fn destruction(current_state: &Self, context: &ValidationContext);
}
