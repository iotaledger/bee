// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    milestone::MilestoneIndex,
    output::{inputs_commitment, ChainId, Output, OutputId, TokenId},
    payload::transaction::{RegularTransactionEssence, TransactionEssence, TransactionId},
    unlock_block::UnlockBlocks,
};

use hashbrown::{HashMap, HashSet};
use primitive_types::U256;

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Errors related to ledger types.
#[derive(Debug, thiserror::Error)]
pub enum ConflictError {
    /// I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Invalid conflict byte.
    #[error("invalid conflict byte")]
    InvalidConflict(u8),
}

/// Represents the different reasons why a transaction can conflict with the ledger state.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, packable::Packable)]
#[packable(unpack_error = ConflictError)]
#[packable(tag_type = u8, with_error = ConflictError::InvalidConflict)]
pub enum ConflictReason {
    /// The message has no conflict.
    None = 0,
    /// The referenced Utxo was already spent.
    InputUtxoAlreadySpent = 1,
    /// The referenced Utxo was already spent while confirming this milestone.
    InputUtxoAlreadySpentInThisMilestone = 2,
    /// The referenced Utxo cannot be found.
    InputUtxoNotFound = 3,
    /// The created amount does not match the consumed amount.
    CreatedConsumedAmountMismatch = 4,
    /// The unlock block signature is invalid.
    InvalidSignature = 5,
    /// The dust allowance for the address is invalid.
    InvalidDustAllowance = 6,
    /// The created native tokens amount does not match the consumed native tokens amount.
    CreatedConsumedNativeTokensAmountMismatch = 7,
    /// The milestone index timelock was no satisfied.
    TimelockMilestoneIndex = 8,
    /// The unix timelock was no satisfied.
    TimelockUnix = 9,
    /// The sender was not verified.
    UnverifiedSender = 10,
    /// An incorrect unlock method was used.
    IncorrectUnlockMethod = 11,
    /// The inputs commitments do not match.
    InputsCommitmentsMismatch = 12,
    /// The semantic validation failed for a reason not covered by the previous variants.
    SemanticValidationFailed = 255,
}

impl Default for ConflictReason {
    fn default() -> Self {
        Self::None
    }
}

impl TryFrom<u8> for ConflictReason {
    type Error = ConflictError;

    fn try_from(c: u8) -> Result<Self, Self::Error> {
        Ok(match c {
            0 => Self::None,
            1 => Self::InputUtxoAlreadySpent,
            2 => Self::InputUtxoAlreadySpentInThisMilestone,
            3 => Self::InputUtxoNotFound,
            4 => Self::CreatedConsumedAmountMismatch,
            5 => Self::InvalidSignature,
            6 => Self::InvalidDustAllowance,
            7 => Self::CreatedConsumedNativeTokensAmountMismatch,
            8 => Self::TimelockMilestoneIndex,
            9 => Self::TimelockUnix,
            10 => Self::UnverifiedSender,
            11 => Self::IncorrectUnlockMethod,
            12 => Self::InputsCommitmentsMismatch,
            255 => Self::SemanticValidationFailed,
            x => return Err(Self::Error::InvalidConflict(x)),
        })
    }
}

///
pub struct ValidationContext<'a> {
    ///
    pub essence: &'a RegularTransactionEssence,
    ///
    pub essence_hash: [u8; 32],
    ///
    pub inputs_commitment: [u8; 32],
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
        inputs: impl Iterator<Item = (&'a OutputId, &'a Output)> + Clone,
        unlock_blocks: &'a UnlockBlocks,
        milestone_index: MilestoneIndex,
        milestone_timestamp: u64,
    ) -> Self {
        Self {
            essence,
            unlock_blocks,
            essence_hash: TransactionEssence::from(essence.clone()).hash(),
            inputs_commitment: inputs_commitment(inputs.clone().map(|(_, output)| output)),
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
