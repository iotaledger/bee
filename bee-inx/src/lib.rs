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
