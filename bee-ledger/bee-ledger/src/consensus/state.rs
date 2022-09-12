// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{output, protocol::ProtocolParameters};
use bee_storage::access::AsIterator;

use crate::{
    error::Error,
    storage::{self, StorageBackend},
    types::Unspent,
};

fn validate_ledger_unspent_state<B: StorageBackend>(
    storage: &B,
    treasury: u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    let iterator = AsIterator::<Unspent, ()>::iter(storage).map_err(|e| Error::Storage(Box::new(e)))?;
    let mut supply: u64 = 0;

    for result in iterator {
        let (output_id, _) = result.map_err(|e| Error::Storage(Box::new(e)))?;
        let output = storage::fetch_output(storage, &*output_id)?.ok_or(Error::MissingUnspentOutput(output_id))?;

        let amount = match output.inner() {
            output::Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(output.kind())),
            output::Output::Basic(output) => output.amount(),
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
        != protocol_parameters.token_supply()
    {
        Err(Error::InvalidLedgerUnspentState(supply))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_ledger_state<B: StorageBackend>(
    storage: &B,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    let treasury = storage::fetch_unspent_treasury_output(storage)?.inner().amount();

    validate_ledger_unspent_state(storage, treasury, protocol_parameters)
}
