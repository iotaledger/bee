// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Bee compatible INX types and INX node request bindings.

#![deny(missing_docs)]

mod error;

/// A module that provides block related requests..
pub mod block;
/// A module that provides the INX client.
pub mod client;
/// A module that provides milestone related requests.
pub mod milestone;
/// A module that provides node related requests.
pub mod node;
/// A module that provides the [`Raw<T: Packable>`] struct.
pub mod raw;
/// A module that provides UTXO ledger related requests.
pub mod utxo;

pub use self::{block::*, error::Error, milestone::*, node::*, raw::*, utxo::*};

pub(crate) mod inx {
    pub use ::inx::proto::{
        block_metadata::*,
        ledger_update::{marker::*, *},
        *,
    };
}

pub(crate) mod bee {
    pub use bee_block::{
        output::{Output, OutputId},
        payload::{
            milestone::{MilestoneId, MilestoneIndex, MilestoneOption},
            transaction::TransactionId,
            Payload,
        },
        protocol::ProtocolParameters,
        semantic::ConflictReason,
        Block, BlockId, InxError,
    };
    #[cfg(test)]
    pub use bee_block::{protocol::protocol_parameters, rand::output::rand_output};
}

#[allow(missing_docs)]
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
    fn test_return_err_if_none() {
        let proto = inx::TreasuryOutput {
            milestone_id: None,
            amount: 42,
        };
        let err = TreasuryOutput::try_from(proto).unwrap_err();
        assert!(matches!(err, bee_block::InxError::MissingField("milestone_id")));
    }
}
