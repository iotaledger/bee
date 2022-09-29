// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// A module that provides utxo related INX responses.
pub mod responses;

use futures::{Stream, StreamExt};

pub use self::responses::*;
use crate::{
    bee,
    client::{from_inx_type, try_from_inx_type, Inx},
    error::Error,
    inx,
    milestone::requests::MilestoneRangeRequest,
    raw::Raw,
};

impl Inx {
    /// Requests all unspent outputs.
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

    /// Creates a feed of ledger updates.
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

    /// Creates a feed of treasury updates.
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

    /// Requests an output by its output id.
    pub async fn read_output(&mut self, output_id: bee::OutputId) -> Result<OutputResponse, Error> {
        Ok(self
            .client
            .read_output(inx::OutputId::from(output_id))
            .await?
            .into_inner()
            .try_into()?)
    }

    /// Creates a feed of migration receipts.
    pub async fn listen_to_migration_receipts(
        &mut self,
    ) -> Result<impl Stream<Item = Result<Raw<bee::MilestoneOption>, Error>>, Error> {
        Ok(self
            .client
            .listen_to_migration_receipts(inx::NoParams {})
            .await?
            .into_inner()
            .map(from_inx_type))
    }
}
