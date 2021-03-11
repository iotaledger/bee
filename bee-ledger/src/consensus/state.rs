// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    consensus::{
        dust::dust_outputs_max,
        error::Error,
        storage::{self, StorageBackend},
    },
    types::{Balance, Unspent},
};

use bee_message::{address::Address, output};
use bee_storage::access::AsStream;

use futures::StreamExt;

// TODO saturating operations ?

pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

async fn check_ledger_unspent_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let mut supply: u64 = 0;
    let mut stream = AsStream::<Unspent, ()>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some((output_id, _)) = stream.next().await {
        // Unwrap: an unspent output has to be in database.
        let output = storage::fetch_output(storage, &*output_id).await?.unwrap();

        match output.inner() {
            output::Output::SignatureLockedSingle(output) => {
                supply += output.amount();
            }
            output::Output::SignatureLockedDustAllowance(output) => {
                supply += output.amount();
            }
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        }
    }

    if supply + treasury != IOTA_SUPPLY {
        return Err(Error::InvalidLedgerUnspentState(supply));
    }

    Ok(())
}

async fn check_ledger_balance_state<B: StorageBackend>(storage: &B, treasury: u64) -> Result<(), Error> {
    let mut supply = 0;
    let mut stream = AsStream::<Address, Balance>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some((address, balance)) = stream.next().await {
        if balance.dust_output() as usize > dust_outputs_max(balance.dust_allowance()) {
            return Err(Error::InvalidLedgerDustState(address, balance));
        }
        supply += balance.amount();
    }

    if supply + treasury != IOTA_SUPPLY {
        return Err(Error::InvalidLedgerBalanceState(supply));
    }

    Ok(())
}

pub async fn check_ledger_state<B: StorageBackend>(storage: &B) -> Result<(), Error> {
    let treasury = storage::fetch_unspent_treasury_output(storage).await?.inner().amount();

    check_ledger_unspent_state(storage, treasury).await?;
    check_ledger_balance_state(storage, treasury).await?;

    Ok(())
}
