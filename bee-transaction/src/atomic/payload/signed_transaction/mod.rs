// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

pub mod input;
mod output;
mod unsigned_transaction;
mod unlock;

pub use unsigned_transaction::UnsignedTransaction;
pub use unlock::UnlockBlock;
use crate::atomic::Error;
use input::Input;

use std::collections::HashSet;

pub struct SignedTransaction {
    pub unsigned_transaction: UnsignedTransaction,
    pub unlock_block_count: u16,
    pub unlock_blocks: Vec<UnlockBlock>
}

impl SignedTransaction {
    pub fn validate(&self) -> Result<(), Error> {
        // Should we add this fiedl? -> Transaction Type value must be 0, denoting an Unsigned Transaction

        // Inputs validation
        let transaction = &self.unsigned_transaction;
        // Inputs Count must be 0 < x < 127
        match transaction.input_count {
            1..=126 => (),
            _ => return Err(Error::CountError)
        }

        // At least one input must be specified
        if transaction.inputs.len() == 0 {
            return Err(Error::EmptyError)
        }

        let mut combination = HashSet::new();
        for i in transaction.inputs.iter() {
            // Input Type value must be 0, denoting an UTXO Input. We use enum in Rust so this is guaranteed to be UTXO.
            match i {
                Input::UTXO(u) => {
                    // Transaction Output Index must be 0 ≤ x < 127
                    match u.output_index {
                        0..=126 => (),
                        _ => return Err(Error::CountError)
                    }

                    // Every combination of Transaction ID + Transaction Output Index must be unique in the inputs set.
                    if combination.insert(u) == false {
                        return Err(Error::DuplicateError);
                    }
                }
            }
        }

        // TODO Inputs must be in lexicographical order of their serialized form.

        // Output validation
        // Outputs Count must be 0 < x < 127
        match transaction.output_count {
            1..=126 => (),
            _ => return Err(Error::CountError)
        }

        // At least one output must be specified
        if transaction.outputs.len() == 0 {
            return Err(Error::EmptyError)
        }

        let mut total = 0;
        for i in transaction.outputs.iter() {
            // Output Type must be 0, denoting a SigLockedSingleDeposit. We use enum in Rust so this is guaranteed to be valid.
            match i {
                output::Output::SigLockedSingleDeposit(u) => {
                    // Address Type must either be 0 or 1, denoting a WOTS- or Ed25519 address. We use enum in Rust so this is guaranteed to be valid.
                    // TODO If Address is of type WOTS address, its bytes must be valid T5B1 bytes

                    // TODO The Address must be unique in the set of SigLockedSingleDeposits

                    // Amount must be > 0
                    if u.amount == 0 {
                        return Err(Error::AmountError);
                    }
                    
                    total += u.amount;
                }
            }
        }

        // TODO Outputs must be in lexicographical order by their serialized form

        // Accumulated output balance must not exceed the total supply of tokens 2'779'530'283'277'761
        if total > 2779530283277761 {
            return Err(Error::AmountError);
        }

        Ok(())
    } 
}
