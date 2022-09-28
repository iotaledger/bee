// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod types;

use futures::{Stream, StreamExt};

pub use self::types::*;
use crate::{
    client::{try_from_inx_type, Inx},
    error::Error,
    inx,
    requests::MilestoneRangeRequest,
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
            .map(try_from_inx_type))
    }

    pub async fn read_unspent_outputs(
        &mut self,
    ) -> Result<impl Stream<Item = Result<crate::UnspentOutput, Error>>, Error> {
        Ok(self
            .client
            .read_unspent_outputs(inx::NoParams {})
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }
}
