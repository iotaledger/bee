// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{Balance, Unspent},
    workers::{
        error::Error,
        storage::{self, StorageBackend},
    },
};

use bee_message::{
    address::Address,
    constants::IOTA_SUPPLY,
    output::{self, dust_outputs_max},
};
use bee_storage::access::AsStream;

use futures::StreamExt;

async fn validate_ledger_unspent_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let mut supply: u64 = 0;
    let mut stream = AsStream::<Unspent, ()>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some(result) = stream.next().await {
        let (output_id, _) = result.map_err(|e| Error::Storage(Box::new(e)))?;
        let output = storage::fetch_output(storage, &*output_id)
            .await?
            .ok_or(Error::MissingUnspentOutput(output_id))?;

        let amount = match output.inner() {
            output::Output::SignatureLockedSingle(output) => output.amount(),
            output::Output::SignatureLockedDustAllowance(output) => output.amount(),
            output::Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(output.kind())),
        };

        supply = supply
            .checked_add(amount)
            .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + amount as u128))?;
    }

    if supply
        .checked_add(treasury)
        .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + treasury as u128))?
        != IOTA_SUPPLY
    {
        Err(Error::InvalidLedgerUnspentState(supply))
    } else {
        Ok(())
    }
}

async fn validate_ledger_balance_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let mut supply: u64 = 0;
    let mut stream = AsStream::<Address, Balance>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some(result) = stream.next().await {
        let (address, balance) = result.map_err(|e| Error::Storage(Box::new(e)))?;
        if balance.dust_outputs() > dust_outputs_max(balance.dust_allowance()) {
            return Err(Error::InvalidLedgerDustState(address, balance));
        }
        supply = supply
            .checked_add(balance.amount())
            .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + balance.amount() as u128))?;
    }

    if supply
        .checked_add(treasury)
        .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + treasury as u128))?
        != IOTA_SUPPLY
    {
        Err(Error::InvalidLedgerBalanceState(supply))
    } else {
        Ok(())
    }
}

pub(crate) async fn validate_ledger_state<B: StorageBackend>(storage: &B) -> Result<(), Error> {
    let treasury = storage::fetch_unspent_treasury_output(storage).await?.inner().amount();

    validate_ledger_unspent_state(storage, treasury).await?;
    validate_ledger_balance_state(storage, treasury).await
}
