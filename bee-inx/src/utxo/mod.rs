// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod types;

use futures::{Stream, StreamExt};

pub use self::types::*;
use crate::{
    client::{try_from_inx_type, Inx},
    error::Error,
    inx,
    milestone::requests::MilestoneRangeRequest,
};

// rpc ReadUnspentOutputs(NoParams) returns (stream UnspentOutput);
// rpc ListenToLedgerUpdates(MilestoneRangeRequest) returns (stream LedgerUpdate);
// rpc ListenToTreasuryUpdates(MilestoneRangeRequest) returns (stream TreasuryUpdate);
// rpc ReadOutput(OutputId) returns (OutputResponse);
// rpc ListenToMigrationReceipts(NoParams) returns (stream RawReceipt);

impl Inx {
    /// TODO
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

    /// TODO
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

    /// TODO
    pub async fn listen_to_treasury_updates(
        &mut self,
        request: MilestoneRangeRequest,
    ) -> Result<impl Stream<Item = Result<TreasuryUpdate, Error>>, Error> {
        Ok(self
            .client
            .listen_to_treasury_updates(inx::MilestoneRangeRequest::from(request))
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }
}
