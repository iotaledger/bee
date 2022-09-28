// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod error;

pub mod block;
pub mod client;
pub mod milestone;
pub mod node;
pub mod raw;
pub mod utxo;

pub use self::{block::*, error::Error, milestone::*, node::*, raw::*, utxo::*};

pub mod inx {
    pub use ::inx::proto::{
        block_metadata::*,
        ledger_update::{marker::*, *},
        *,
    };
}

pub(crate) mod bee {
    pub use bee_block::{payload::milestone::MilestoneIndex, protocol::ProtocolParameters, InxError};
    pub use bee_block::{Block, BlockId};
    pub use bee_block::semantic::ConflictReason;
}

#[macro_export]
macro_rules! return_err_if_none {
    ($object:ident.$field:ident) => {
        $object.$field.ok_or(Self::Error::MissingField(stringify!($field)))?
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn macro_missing_field() {
        let proto = inx::TreasuryOutput {
            milestone_id: None,
            amount: 42,
        };
        let err = TreasuryOutput::try_from(proto).unwrap_err();
        assert!(matches!(err, bee_block::InxError::MissingField("milestone_id")));
    }
}
