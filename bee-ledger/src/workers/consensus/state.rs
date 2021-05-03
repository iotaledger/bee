// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{Balance, Unspent},
    workers::{
        consensus::dust::dust_outputs_max,
        error::Error,
        storage::{self, StorageBackend},
    },
};

use bee_message::{address::Address, constants::IOTA_SUPPLY, output};
use bee_storage::access::AsStream;

use futures::StreamExt;

async fn validate_ledger_unspent_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let mut supply: u64 = 0;
    let mut stream = AsStream::<Unspent, ()>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some((output_id, _)) = stream.next().await {
        // Unwrap: an unspent output has to be in database.
        let output = storage::fetch_output(storage, &*output_id).await?.unwrap();

        match output.inner() {
            output::Output::SignatureLockedSingle(output) => {
                supply = supply
                    .checked_add(output.amount())
                    .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + output.amount() as u128))?;
            }
            output::Output::SignatureLockedDustAllowance(output) => {
                supply = supply
                    .checked_add(output.amount())
                    .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + output.amount() as u128))?;
            }
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        }
    }

    if supply
        .checked_add(treasury)
        .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + treasury as u128))?
        != IOTA_SUPPLY
    {
        return Err(Error::InvalidLedgerUnspentState(supply));
    }

    Ok(())
}

async fn validate_ledger_balance_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let mut supply: u64 = 0;
    let mut stream = AsStream::<Address, Balance>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some((address, balance)) = stream.next().await {
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
        return Err(Error::InvalidLedgerBalanceState(supply));
    }

    Ok(())
}

pub(crate) async fn validate_ledger_state<B: StorageBackend>(storage: &B) -> Result<(), Error> {
    let treasury = storage::fetch_unspent_treasury_output(storage).await?.inner().amount();

    validate_ledger_unspent_state(storage, treasury).await?;
    validate_ledger_balance_state(storage, treasury).await
}
