// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod block;
pub mod client;
mod error;
mod ledger;
mod metadata;
mod milestone;
mod node;
mod raw;
mod request;
mod treasury;

pub use self::{
    block::*, error::Error, ledger::*, metadata::*, milestone::*, node::*, raw::*, request::*, treasury::*,
};

pub mod proto {
    pub use inx::proto::*;
}

#[macro_export]
macro_rules! field {
    ($object:ident.$field:ident) => {
        $object.$field.ok_or(Self::Error::MissingField(stringify!($field)))?
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn macro_missing_field() {
        let proto = proto::TreasuryOutput{
            milestone_id: None,
            amount: 42,
        };
        let err = TreasuryOutput::try_from(proto).unwrap_err();
        assert!(matches!(err, bee_block::InxError::MissingField("milestone_id")));
    }
}
