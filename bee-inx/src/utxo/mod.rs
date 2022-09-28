// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod ledger;
pub mod treasury;

use futures::{Stream, StreamExt};

pub use self::{ledger::*, treasury::*};
use crate::{
    client::{try_convert_proto_msg, Inx},
    error::Error,
    inx,
    request::MilestoneRangeRequest,
};

impl Inx {
    pub async fn listen_to_ledger_updates(
        &mut self,
        request: MilestoneRangeRequest,
    ) -> Result<impl Stream<Item = Result<LedgerUpdate, Error>>, Error> {
        Ok(self
            .client
            .listen_to_ledger_updates(inx::MilestoneRangeRequest::from(request))
            .await?
            .into_inner()
            .map(try_convert_proto_msg))
    }

    pub async fn read_unspent_outputs(
        &mut self,
    ) -> Result<impl Stream<Item = Result<crate::UnspentOutput, Error>>, Error> {
        Ok(self
            .client
            .read_unspent_outputs(inx::NoParams {}) // TODO: ().into()
            .await?
            .into_inner()
            .map(try_convert_proto_msg))
    }
}