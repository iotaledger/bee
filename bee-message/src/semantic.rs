// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::{convert::Infallible, fmt};

use hashbrown::{HashMap, HashSet};
use primitive_types::U256;

use crate::{
    address::Address,
    error::Error,
    milestone::MilestoneIndex,
    output::{create_inputs_commitment, ChainId, NativeTokens, Output, OutputId, TokenId, UnlockCondition},
    payload::transaction::{RegularTransactionEssence, TransactionEssence, TransactionId},
    unlock_block::UnlockBlocks,
};

/// Errors related to ledger types.
#[derive(Debug)]
pub enum ConflictError {
    /// Invalid conflict byte.
    InvalidConflict(u8),
}

impl fmt::Display for ConflictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConflictError::InvalidConflict(byte) => write!(f, "invalid conflict byte {byte}"),
        }
    }
}

impl From<Infallible> for ConflictError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConflictError {}

/// Represents the different reasons why a transaction can conflict with the ledger state.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Too many native tokens.
    TooManyNativeTokens = 15,
    /// Non unique token id for founfry.
    NonUniqueTokenIdForFoundry = 16,
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
            15 => Self::TooManyNativeTokens,
            16 => Self::NonUniqueTokenIdForFoundry,
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

///
pub fn semantic_validation(
    mut context: ValidationContext,
    inputs: &[(OutputId, &Output)],
    unlock_blocks: &UnlockBlocks,
) -> Result<ConflictReason, Error> {
    // Validation of the inputs commitment.
    if context.essence.inputs_commitment() != &context.inputs_commitment {
        return Ok(ConflictReason::InputsCommitmentsMismatch);
    }

    // Validation of inputs.
    for ((output_id, consumed_output), unlock_block) in inputs.iter().zip(unlock_blocks.iter()) {
        let (conflict, amount, consumed_native_tokens, unlock_conditions) = match consumed_output {
            Output::Basic(output) => (
                output.unlock(output_id, unlock_block, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Alias(output) => (
                output.unlock(output_id, unlock_block, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Foundry(output) => (
                output.unlock(output_id, unlock_block, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Nft(output) => (
                output.unlock(output_id, unlock_block, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            _ => return Err(Error::UnsupportedOutputKind(consumed_output.kind())),
        };

        if let Err(conflict) = conflict {
            return Ok(conflict);
        }

        if let Some(timelock) = unlock_conditions.timelock() {
            if *timelock.milestone_index() != 0 && context.milestone_index < timelock.milestone_index() {
                return Ok(ConflictReason::TimelockMilestoneIndex);
            }
            if timelock.timestamp() != 0 && context.milestone_timestamp < timelock.timestamp() as u64 {
                return Ok(ConflictReason::TimelockUnix);
            }
        }

        if let Some(storage_deposit_return) = unlock_conditions.storage_deposit_return() {
            let amount = context
                .storage_deposit_returns
                .entry(*storage_deposit_return.return_address())
                .or_default();

            *amount = amount
                .checked_add(storage_deposit_return.amount())
                .ok_or(Error::StorageDepositReturnOverflow)?;
        }

        context.input_amount = context
            .input_amount
            .checked_add(amount)
            .ok_or(Error::ConsumedAmountOverflow)?;

        for native_token in consumed_native_tokens.iter() {
            let native_token_amount = context.input_native_tokens.entry(*native_token.token_id()).or_default();

            *native_token_amount = native_token_amount
                .checked_add(*native_token.amount())
                .ok_or(Error::ConsumedNativeTokensAmountOverflow)?;
        }
    }

    // Validation of outputs.
    for created_output in context.essence.outputs() {
        let (amount, created_native_tokens, feature_blocks) = match created_output {
            Output::Basic(output) => {
                if let [UnlockCondition::Address(address)] = output.unlock_conditions().as_ref() {
                    if output.feature_blocks().is_empty() {
                        let amount = context.simple_deposits.entry(*address.address()).or_default();

                        *amount = amount
                            .checked_add(output.amount())
                            .ok_or(Error::CreatedAmountOverflow)?;
                    }
                }
                (output.amount(), output.native_tokens(), output.feature_blocks())
            }
            Output::Alias(output) => (output.amount(), output.native_tokens(), output.feature_blocks()),
            Output::Foundry(output) => (output.amount(), output.native_tokens(), output.feature_blocks()),
            Output::Nft(output) => (output.amount(), output.native_tokens(), output.feature_blocks()),
            _ => return Err(Error::UnsupportedOutputKind(created_output.kind())),
        };

        if let Some(sender) = feature_blocks.sender() {
            if !context.unlocked_addresses.contains(sender.address()) {
                return Ok(ConflictReason::UnverifiedSender);
            }
        }

        context.output_amount = context
            .output_amount
            .checked_add(amount)
            .ok_or(Error::CreatedAmountOverflow)?;

        for native_token in created_native_tokens.iter() {
            let native_token_amount = context
                .output_native_tokens
                .entry(*native_token.token_id())
                .or_default();

            *native_token_amount = native_token_amount
                .checked_add(*native_token.amount())
                .ok_or(Error::CreatedNativeTokensAmountOverflow)?;
        }
    }

    // Validation of storage deposit returns.
    for (return_address, return_amount) in context.storage_deposit_returns.iter() {
        if let Some(deposit_amount) = context.simple_deposits.get(return_address) {
            if deposit_amount < return_amount {
                return Ok(ConflictReason::StorageDepositReturnMismatch);
            }
        } else {
            return Ok(ConflictReason::StorageDepositReturnMismatch);
        }
    }

    // Validation of amounts.
    if context.input_amount != context.output_amount {
        return Ok(ConflictReason::CreatedConsumedAmountMismatch);
    }

    let mut native_token_ids = HashMap::new();

    // Validation of input native tokens.
    for (token_id, input_amount) in context.input_native_tokens.iter() {
        let output_amount = context.output_native_tokens.get(token_id).copied().unwrap_or_default();

        if input_amount > &output_amount
            && !context
                .output_chains
                .contains_key(&ChainId::from(token_id.foundry_id()))
        {
            return Ok(ConflictReason::CreatedConsumedNativeTokensAmountMismatch);
        }

        if let Some(token_tag) = native_token_ids.insert(token_id.foundry_id(), token_id.token_tag()) {
            if token_tag != token_id.token_tag() {
                return Ok(ConflictReason::NonUniqueTokenIdForFoundry);
            }
        }
    }

    // Validation of output native tokens.
    for (token_id, output_amount) in context.output_native_tokens.iter() {
        let input_amount = context.input_native_tokens.get(token_id).copied().unwrap_or_default();

        if output_amount > &input_amount
            && !context
                .output_chains
                .contains_key(&ChainId::from(token_id.foundry_id()))
        {
            return Ok(ConflictReason::CreatedConsumedNativeTokensAmountMismatch);
        }

        if let Some(token_tag) = native_token_ids.insert(token_id.foundry_id(), token_id.token_tag()) {
            if token_tag != token_id.token_tag() {
                return Ok(ConflictReason::NonUniqueTokenIdForFoundry);
            }
        }
    }

    if native_token_ids.len() > NativeTokens::COUNT_MAX as usize {
        return Ok(ConflictReason::TooManyNativeTokens);
    }

    // Validation of state transitions and destructions.
    for (chain_id, current_state) in context.input_chains.iter() {
        if Output::verify_state_transition(
            Some(current_state),
            context.output_chains.get(chain_id).map(core::ops::Deref::deref),
            &context,
        )
        .is_err()
        {
            return Ok(ConflictReason::SemanticValidationFailed);
        }
    }

    // Validation of state creations.
    for (chain_id, next_state) in context.output_chains.iter() {
        if context.input_chains.get(chain_id).is_none()
            && Output::verify_state_transition(None, Some(next_state), &context).is_err()
        {
            return Ok(ConflictReason::SemanticValidationFailed);
        }
    }

    Ok(ConflictReason::None)
}
