// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::{convert::Infallible, fmt};

use hashbrown::{HashMap, HashSet};
use primitive_types::U256;

use crate::{
    address::Address,
    error::Error,
    output::{ChainId, FoundryId, InputsCommitment, NativeTokens, Output, OutputId, TokenId},
    payload::transaction::{RegularTransactionEssence, TransactionEssence, TransactionId},
    unlock::Unlocks,
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
    /// The block has no conflict.
    None = 0,
    /// The referenced Utxo was already spent.
    InputUtxoAlreadySpent = 1,
    /// The referenced Utxo was already spent while confirming this milestone.
    InputUtxoAlreadySpentInThisMilestone = 2,
    /// The referenced Utxo cannot be found.
    InputUtxoNotFound = 3,
    /// The created amount does not match the consumed amount.
    CreatedConsumedAmountMismatch = 4,
    /// The unlock signature is invalid.
    InvalidSignature = 5,
    /// The configured timelock is not yet expired.
    TimelockNotExpired = 6,
    /// The given native tokens are invalid.
    InvalidNativeTokens = 7,
    /// Storage deposit return mismatch.
    StorageDepositReturnUnfulfilled = 8,
    /// An invalid unlock was used.
    InvalidUnlock = 9,
    /// The inputs commitments do not match.
    InputsCommitmentsMismatch = 10,
    /// The sender was not verified.
    UnverifiedSender = 11,
    /// The chain state transition is invalid.
    InvalidChainStateTransition = 12,
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
            6 => Self::TimelockNotExpired,
            7 => Self::InvalidNativeTokens,
            8 => Self::StorageDepositReturnUnfulfilled,
            9 => Self::InvalidUnlock,
            10 => Self::InputsCommitmentsMismatch,
            11 => Self::UnverifiedSender,
            12 => Self::InvalidChainStateTransition,
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
    pub inputs_commitment: InputsCommitment,
    ///
    pub unlocks: &'a Unlocks,
    ///
    pub milestone_timestamp: u32,
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
        unlocks: &'a Unlocks,
        milestone_timestamp: u32,
    ) -> Self {
        Self {
            essence,
            unlocks,
            essence_hash: TransactionEssence::from(essence.clone()).hash(),
            inputs_commitment: InputsCommitment::new(inputs.clone().map(|(_, output)| output)),
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
    unlocks: &Unlocks,
) -> Result<ConflictReason, Error> {
    // Validation of the inputs commitment.
    if context.essence.inputs_commitment() != &context.inputs_commitment {
        return Ok(ConflictReason::InputsCommitmentsMismatch);
    }

    // Validation of inputs.
    for ((output_id, consumed_output), unlock) in inputs.iter().zip(unlocks.iter()) {
        let (conflict, amount, consumed_native_tokens, unlock_conditions) = match consumed_output {
            Output::Basic(output) => (
                output.unlock(output_id, unlock, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Alias(output) => (
                output.unlock(output_id, unlock, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Foundry(output) => (
                output.unlock(output_id, unlock, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Nft(output) => (
                output.unlock(output_id, unlock, inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            _ => return Err(Error::UnsupportedOutputKind(consumed_output.kind())),
        };

        if let Err(conflict) = conflict {
            return Ok(conflict);
        }

        if unlock_conditions.is_time_locked(context.milestone_timestamp) {
            return Ok(ConflictReason::TimelockNotExpired);
        }

        if !unlock_conditions.is_expired(context.milestone_timestamp) {
            if let Some(storage_deposit_return) = unlock_conditions.storage_deposit_return() {
                let amount = context
                    .storage_deposit_returns
                    .entry(*storage_deposit_return.return_address())
                    .or_default();

                *amount = amount
                    .checked_add(storage_deposit_return.amount())
                    .ok_or(Error::StorageDepositReturnOverflow)?;
            }
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
        let (amount, created_native_tokens, features) = match created_output {
            Output::Basic(output) => {
                if let Some(address) = output.simple_deposit_address() {
                    let amount = context.simple_deposits.entry(*address).or_default();

                    *amount = amount
                        .checked_add(output.amount())
                        .ok_or(Error::CreatedAmountOverflow)?;
                }

                (output.amount(), output.native_tokens(), output.features())
            }
            Output::Alias(output) => (output.amount(), output.native_tokens(), output.features()),
            Output::Foundry(output) => (output.amount(), output.native_tokens(), output.features()),
            Output::Nft(output) => (output.amount(), output.native_tokens(), output.features()),
            _ => return Err(Error::UnsupportedOutputKind(created_output.kind())),
        };

        if let Some(sender) = features.sender() {
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
                return Ok(ConflictReason::StorageDepositReturnUnfulfilled);
            }
        } else {
            return Ok(ConflictReason::StorageDepositReturnUnfulfilled);
        }
    }

    // Validation of amounts.
    if context.input_amount != context.output_amount {
        return Ok(ConflictReason::CreatedConsumedAmountMismatch);
    }

    let mut native_token_ids = HashSet::new();

    // Validation of input native tokens.
    for (token_id, _input_amount) in context.input_native_tokens.iter() {
        native_token_ids.insert(token_id);
    }

    // Validation of output native tokens.
    for (token_id, output_amount) in context.output_native_tokens.iter() {
        let input_amount = context.input_native_tokens.get(token_id).copied().unwrap_or_default();

        if output_amount > &input_amount
            && !context
                .output_chains
                .contains_key(&ChainId::from(FoundryId::from(*token_id)))
        {
            return Ok(ConflictReason::InvalidNativeTokens);
        }

        native_token_ids.insert(token_id);
    }

    if native_token_ids.len() > NativeTokens::COUNT_MAX as usize {
        return Ok(ConflictReason::InvalidNativeTokens);
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
            return Ok(ConflictReason::InvalidChainStateTransition);
        }
    }

    // Validation of state creations.
    for (chain_id, next_state) in context.output_chains.iter() {
        if context.input_chains.get(chain_id).is_none()
            && Output::verify_state_transition(None, Some(next_state), &context).is_err()
        {
            return Ok(ConflictReason::InvalidChainStateTransition);
        }
    }

    Ok(ConflictReason::None)
}

#[cfg(feature = "inx")]
mod inx {
    use super::*;

    impl From<inx_bindings::proto::block_metadata::ConflictReason> for ConflictReason {
        fn from(value: inx_bindings::proto::block_metadata::ConflictReason) -> Self {
            use inx_bindings::proto::block_metadata::ConflictReason as InxConflictReason;
            match value {
                InxConflictReason::None => ConflictReason::None,
                InxConflictReason::InputAlreadySpent => ConflictReason::InputUtxoAlreadySpent,
                InxConflictReason::InputAlreadySpentInThisMilestone => {
                    ConflictReason::InputUtxoAlreadySpentInThisMilestone
                }
                InxConflictReason::InputNotFound => ConflictReason::InputUtxoNotFound,
                InxConflictReason::InputOutputSumMismatch => ConflictReason::CreatedConsumedAmountMismatch,
                InxConflictReason::InvalidSignature => ConflictReason::InvalidSignature,
                InxConflictReason::TimelockNotExpired => ConflictReason::TimelockNotExpired,
                InxConflictReason::InvalidNativeTokens => ConflictReason::InvalidNativeTokens,
                InxConflictReason::ReturnAmountNotFulfilled => ConflictReason::StorageDepositReturnUnfulfilled,
                InxConflictReason::InvalidInputUnlock => ConflictReason::InvalidUnlock,
                InxConflictReason::InvalidInputsCommitment => ConflictReason::InputsCommitmentsMismatch,
                InxConflictReason::InvalidSender => ConflictReason::UnverifiedSender,
                InxConflictReason::InvalidChainStateTransition => ConflictReason::InvalidChainStateTransition,
                InxConflictReason::SemanticValidationFailed => ConflictReason::SemanticValidationFailed,
            }
        }
    }

    impl From<ConflictReason> for inx_bindings::proto::block_metadata::ConflictReason {
        fn from(value: ConflictReason) -> Self {
            match value {
                ConflictReason::None => Self::None,
                ConflictReason::InputUtxoAlreadySpent => Self::InputAlreadySpent,
                ConflictReason::InputUtxoAlreadySpentInThisMilestone => Self::InputAlreadySpentInThisMilestone,
                ConflictReason::InputUtxoNotFound => Self::InputNotFound,
                ConflictReason::CreatedConsumedAmountMismatch => Self::InputOutputSumMismatch,
                ConflictReason::InvalidSignature => Self::InvalidSignature,
                ConflictReason::TimelockNotExpired => Self::TimelockNotExpired,
                ConflictReason::InvalidNativeTokens => Self::InvalidNativeTokens,
                ConflictReason::StorageDepositReturnUnfulfilled => Self::ReturnAmountNotFulfilled,
                ConflictReason::InvalidUnlock => Self::InvalidInputUnlock,
                ConflictReason::InputsCommitmentsMismatch => Self::InvalidInputsCommitment,
                ConflictReason::UnverifiedSender => Self::InvalidSender,
                ConflictReason::InvalidChainStateTransition => Self::InvalidChainStateTransition,
                ConflictReason::SemanticValidationFailed => Self::SemanticValidationFailed,
            }
        }
    }
}
