// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::semantic::ValidationContext;

///
#[allow(missing_docs)]
pub enum StateTransitionError {
    MutatedFieldWithoutRights,
    MutatedImmutableField,
    NonZeroCreatedId,
    UnsupportedStateIndexOperation { current_state: u32, next_state: u32 },
}

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
