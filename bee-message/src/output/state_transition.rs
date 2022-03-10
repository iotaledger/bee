// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::semantic::ValidationContext;

///
pub enum StateTransitionError {}

///
pub trait StateTransition {
    ///
    fn creation(next_state: &Self, context: &ValidationContext) -> Result<(), StateTransitionError>;

    ///
    fn transition(
        current_state: &Self,
        next_state: &Self,
        context: &ValidationContext,
    ) -> Result<(), StateTransitionError>;

    ///
    fn destruction(current_state: &Self, context: &ValidationContext) -> Result<(), StateTransitionError>;
}
