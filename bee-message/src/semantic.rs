// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    milestone::MilestoneIndex,
    output::{ChainId, Output, OutputId, TokenId},
    payload::transaction::{RegularTransactionEssence, TransactionEssence, TransactionId},
    unlock_block::UnlockBlocks,
};

use hashbrown::{HashMap, HashSet};
use primitive_types::U256;

///
pub struct ValidationContext<'a> {
    ///
    pub essence: &'a RegularTransactionEssence,
    ///
    pub essence_hash: [u8; 32],
    ///
    pub unlock_blocks: &'a UnlockBlocks,
    ///
    pub milestone_index: MilestoneIndex,
    ///
    pub milestone_timestamp: u64,
    ///
    pub input_amount: u64,
    ///
    pub input_native_tokens: HashMap<TokenId, U256>,
    ///
    pub input_chains: HashMap<ChainId, &'a Output>,
    ///
    pub output_amount: u64,
    ///
    pub output_native_tokens: HashMap<TokenId, U256>,
    ///
    pub output_chains: HashMap<ChainId, &'a Output>,
    ///
    pub unlocked_addresses: HashSet<Address>,
}

impl<'a> ValidationContext<'a> {
    ///
    pub fn new(
        transaction_id: &TransactionId,
        essence: &'a RegularTransactionEssence,
        inputs: impl Iterator<Item = (&'a OutputId, &'a Output)>,
        unlock_blocks: &'a UnlockBlocks,
        milestone_index: MilestoneIndex,
        milestone_timestamp: u64,
    ) -> Self {
        Self {
            essence,
            unlock_blocks,
            essence_hash: TransactionEssence::from(essence.clone()).hash(),
            milestone_index,
            milestone_timestamp,
            input_amount: 0,
            input_native_tokens: HashMap::<TokenId, U256>::new(),
            input_chains: inputs
                .filter_map(|(output_id, input)| {
                    input
                        .chain_id()
                        .map(|chain_id| (chain_id.or_from_output_id(*output_id), input))
                })
                .collect(),
            output_amount: 0,
            output_native_tokens: HashMap::<TokenId, U256>::new(),
            output_chains: essence
                .outputs()
                .iter()
                .enumerate()
                .filter_map(|(index, output)| {
                    output.chain_id().map(|chain_id| {
                        (
                            chain_id.or_from_output_id(OutputId::new(*transaction_id, index as u16).unwrap()),
                            output,
                        )
                    })
                })
                .collect(),
            unlocked_addresses: HashSet::new(),
        }
    }
}
