// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    milestone::MilestoneIndex,
    output::{create_inputs_commitment, ChainId, Output, OutputId, TokenId},
    payload::transaction::{RegularTransactionEssence, TransactionEssence, TransactionId},
    unlock_block::UnlockBlocks,
};

use hashbrown::{HashMap, HashSet};
use primitive_types::U256;

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
    /// The created native tokens amount does not match the consumed native tokens amount.
    CreatedConsumedNativeTokensAmountMismatch = 6,
    /// The milestone index timelock was no satisfied.
    TimelockMilestoneIndex = 7,
    /// The unix timelock was no satisfied.
    TimelockUnix = 8,
    /// The sender was not verified.
    UnverifiedSender = 9,
    /// An incorrect unlock method was used.
    IncorrectUnlockMethod = 10,
    /// The inputs commitments do not match.
    InputsCommitmentsMismatch = 11,
    /// Storage deposit return mismatch.
    StorageDepositReturnMismatch = 12,
    /// Unlock and address types mismatch.
    UnlockAddressMismatch = 13,
    /// The address was not previously unlocked.
    AddressNotUnlocked = 14,
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
            6 => Self::CreatedConsumedNativeTokensAmountMismatch,
            7 => Self::TimelockMilestoneIndex,
            8 => Self::TimelockUnix,
            9 => Self::UnverifiedSender,
            10 => Self::IncorrectUnlockMethod,
            11 => Self::InputsCommitmentsMismatch,
            12 => Self::StorageDepositReturnMismatch,
            13 => Self::UnlockAddressMismatch,
            14 => Self::AddressNotUnlocked,
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
    ///
    pub storage_deposit_returns: HashMap<Address, u64>,
    ///
    pub simple_deposits: HashMap<Address, u64>,
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
            inputs_commitment: create_inputs_commitment(inputs.clone().map(|(_, output)| output)),
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
            storage_deposit_returns: HashMap::new(),
            simple_deposits: HashMap::new(),
        }
    }
}
