// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    balance::Balance,
    error::Error,
    model::Unspent,
    storage::{self, StorageBackend},
};

use bee_message::payload::transaction::{self, Address};
use bee_storage::access::AsStream;

use futures::StreamExt;

pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

async fn check_ledger_unspent_state<B: StorageBackend>(storage: &B) -> Result<bool, Error> {
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

    Ok(supply == IOTA_SUPPLY)
}

async fn check_ledger_balance_state<B: StorageBackend>(storage: &B) -> Result<bool, Error> {
    let stream = AsStream::<Address, Balance>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    let supply = stream
        .fold(0, |acc, (_, balance)| async move { acc + balance.amount() })
        .await;

    Ok(supply == IOTA_SUPPLY)
}

pub async fn check_ledger_state<B: StorageBackend>(storage: &B) -> Result<bool, Error> {
    Ok(check_ledger_unspent_state(storage).await? && check_ledger_balance_state(storage).await?)
}
