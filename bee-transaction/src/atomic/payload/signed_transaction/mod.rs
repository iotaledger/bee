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
mod unlock;
mod unsigned_transaction;

use crate::atomic::Error;
use input::Input;
pub use unlock::UnlockBlock;
pub use unsigned_transaction::UnsignedTransaction;

use std::collections::HashSet;

pub struct SignedTransaction {
    pub unsigned_transaction: UnsignedTransaction,
    pub unlock_block_count: u16,
    pub unlock_blocks: Vec<UnlockBlock>,
}

impl SignedTransaction {
    pub fn validate(&self) -> Result<(), Error> {
        // Should we add this fiedl? -> Transaction Type value must be 0, denoting an Unsigned Transaction

        // Inputs validation
        let transaction = &self.unsigned_transaction;
        // Inputs Count must be 0 < x < 127
        match transaction.input_count {
            1..=126 => (),
            _ => return Err(Error::CountError),
        }

        // At least one input must be specified
        if transaction.inputs.len() == 0 {
            return Err(Error::EmptyError);
        }

        let mut combination = HashSet::new();
        for i in transaction.inputs.iter() {
            // Input Type value must be 0, denoting an UTXO Input.
            match i {
                Input::UTXO(u) => {
                    // Transaction Output Index must be 0 ≤ x < 127
                    match u.output_index {
                        0..=126 => (),
                        _ => return Err(Error::CountError),
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
            _ => return Err(Error::CountError),
        }

        // At least one output must be specified
        if transaction.outputs.len() == 0 {
            return Err(Error::EmptyError);
        }

        let mut total = 0;
        for i in transaction.outputs.iter() {
            // Output Type must be 0, denoting a SigLockedSingleDeposit.
            match i {
                output::Output::SigLockedSingleDeposit(u) => {
                    // Address Type must either be 0 or 1, denoting a WOTS- or Ed25519 address.
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

        // Payload Length must be 0 (to indicate that there's no payload) or be valid for the specified payload type.
        // Payload Type must be one of the supported payload types if Payload Length is not 0.

        // Unlock Blocks validation
        // Unlock Blocks Count must match the amount of inputs. Must be 0 < x < 127.
        match self.unlock_block_count {
            1..=126 => (),
            _ => return Err(Error::CountError),
        }

        // Unlock Block Type must either be 0 or 1, denoting a Signature Unlock Block or Reference Unlock block.
        let mut combination = HashSet::new();
        for (i, block) in self.unlock_blocks.iter().enumerate() {
            // Signature Unlock Blocks must define either an Ed25519- or WOTS Signature
            match block {
                UnlockBlock::Reference(r) => {
                    // Reference Unlock Blocks must specify a previous Unlock Block which is not of type Reference
                    // Unlock Block. Since it's not the first input it unlocks, it must have
                    // differente transaction id from previous one
                    if i != 0 {
                        match &transaction.inputs[i] {
                            Input::UTXO(u) => match &transaction.inputs[i - 1] {
                                Input::UTXO(v) => {
                                    if u.transaction_id != v.transaction_id {
                                        return Err(Error::IndexError);
                                    }
                                }
                            },
                        }
                    }

                    // The reference index must therefore be < the index of the Reference Unlock Block
                    if r.index >= i as u16 {
                        return Err(Error::IndexError);
                    }
                }
                UnlockBlock::Signature(s) => {
                    // A Signature Unlock Block unlocking multiple inputs must only appear once (be unique) and be
                    // positioned at same index of the first input it unlocks.
                    if combination.insert(s) == false {
                        return Err(Error::DuplicateError);
                    }

                    // Since it's first input it unlocks, it must have differente transaction id from previous one
                    if i != 0 {
                        match &transaction.inputs[i] {
                            Input::UTXO(u) => match &transaction.inputs[i - 1] {
                                Input::UTXO(v) => {
                                    if u.transaction_id == v.transaction_id {
                                        return Err(Error::IndexError);
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }

        // TODO Semantic Validation

        Ok(())
    }
}
