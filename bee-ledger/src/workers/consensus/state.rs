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
    constant::IOTA_SUPPLY,
    output::{self},
};
use bee_storage::access::AsIterator;

fn validate_ledger_unspent_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let iterator = AsIterator::<Unspent, ()>::iter(storage).map_err(|e| Error::Storage(Box::new(e)))?;
    let mut supply: u64 = 0;

    for result in iterator {
        let (output_id, _) = result.map_err(|e| Error::Storage(Box::new(e)))?;
        let output = storage::fetch_output(storage, &*output_id)?.ok_or(Error::MissingUnspentOutput(output_id))?;

        let amount = match output.inner() {
            output::Output::Simple(output) => output.amount(),
            output::Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(output.kind())),
            output::Output::Extended(output) => output.amount(),
            output::Output::Alias(output) => output.amount(),
            output::Output::Foundry(output) => output.amount(),
            output::Output::Nft(output) => output.amount(),
        };

        supply = supply
            .checked_add(amount)
            .ok_or(Error::LedgerStateOverflow(supply as u128 + amount as u128))?;
    }

    if supply
        .checked_add(treasury)
        .ok_or(Error::LedgerStateOverflow(supply as u128 + treasury as u128))?
        != IOTA_SUPPLY
    {
        Err(Error::InvalidLedgerUnspentState(supply))
    } else {
        Ok(())
    }
}

fn validate_ledger_balance_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let iterator = AsIterator::<Address, Balance>::iter(storage).map_err(|e| Error::Storage(Box::new(e)))?;
    let mut supply: u64 = 0;

    for result in iterator {
        let (_, balance) = result.map_err(|e| Error::Storage(Box::new(e)))?;
        // TODO check dust ?
        supply = supply
            .checked_add(balance.amount())
            .ok_or_else(|| Error::LedgerStateOverflow(supply as u128 + balance.amount() as u128))?;
    }

    if supply
        .checked_add(treasury)
        .ok_or(Error::LedgerStateOverflow(supply as u128 + treasury as u128))?
        != IOTA_SUPPLY
    {
        Err(Error::InvalidLedgerBalanceState(supply))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_ledger_state<B: StorageBackend>(storage: &B) -> Result<(), Error> {
    let treasury = storage::fetch_unspent_treasury_output(storage)?.inner().amount();

    validate_ledger_unspent_state(storage, treasury)?;
    validate_ledger_balance_state(storage, treasury)
}
