// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    balance::Balance,
    dust::dust_outputs_max,
    error::Error,
    model::Unspent,
    storage::{self, StorageBackend},
};

use bee_message::payload::transaction::{self, Address};
use bee_storage::access::AsStream;

use futures::StreamExt;

pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

async fn check_ledger_unspent_state<B: StorageBackend>(storage: &B) -> Result<(), Error> {
    let mut supply: u64 = 0;
    let mut stream = AsStream::<Unspent, ()>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some((output_id, _)) = stream.next().await {
        // Unwrap: an unspent output has to be in database.
        let output = storage::fetch_output(storage, &*output_id).await?.unwrap();

        match output.inner() {
            transaction::Output::SignatureLockedSingle(output) => {
                supply += output.amount();
            }
            transaction::Output::SignatureLockedDustAllowance(output) => {
                supply += output.amount();
            }
            _ => return Err(Error::UnsupportedOutputType),
        }
    }

    if supply != IOTA_SUPPLY {
        return Err(Error::InvalidLedgerUnspentState(supply));
    }

    Ok(())
}

async fn check_ledger_balance_state<B: StorageBackend>(storage: &B) -> Result<(), Error> {
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

    if supply != IOTA_SUPPLY {
        return Err(Error::InvalidLedgerBalanceState(supply));
    }

    Ok(())
}

pub async fn check_ledger_state<B: StorageBackend>(storage: &B) -> Result<(), Error> {
    check_ledger_unspent_state(storage).await?;
    check_ledger_balance_state(storage).await?;

    Ok(())
}
