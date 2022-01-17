// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    input::{Input, INPUT_COUNT_RANGE},
    output::{Output, SignatureLockedSingleOutput, OUTPUT_COUNT_RANGE},
    payload::Payload,
    MessageUnpackError, ValidationError, IOTA_SUPPLY,
};

use bee_ord::is_sorted;

use packable::{bounded::BoundedU32, prefix::VecPrefix, Packable, PackableExt};

use alloc::vec::Vec;
use core::{convert::Infallible, fmt};

/// Length (in bytes) of Transaction Essence pledge IDs (node IDs relating to pledge mana).
pub const PLEDGE_ID_LENGTH: usize = 32;

pub(crate) type InputCount = BoundedU32<{ *INPUT_COUNT_RANGE.start() as u32 }, { *INPUT_COUNT_RANGE.end() as u32 }>;
pub(crate) type OutputCount = BoundedU32<{ *OUTPUT_COUNT_RANGE.start() as u32 }, { *OUTPUT_COUNT_RANGE.end() as u32 }>;

/// Error encountered unpacking a Transaction Essence.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum TransactionEssenceUnpackError {
    InvalidOutputKind(u8),
    InvalidOptionTag(u8),
    Validation(ValidationError),
}

impl_wrapped_variant!(
    TransactionEssenceUnpackError,
    TransactionEssenceUnpackError::Validation,
    ValidationError
);
impl_from_infallible!(TransactionEssenceUnpackError);

impl fmt::Display for TransactionEssenceUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOutputKind(kind) => write!(f, "invalid output kind: {}", kind),
            Self::InvalidOptionTag(tag) => write!(f, "invalid tag for Option: {} is not 0 or 1", tag),
            Self::Validation(e) => write!(f, "{}", e),
        }
    }
}

/// A [`TransactionEssence`] consuming [`Input`]s, creating [`Output]`s and carrying an optional [`Payload`].
///
/// A [`TransactionEssence`] must:
/// * Contain a number of [`Input`]s within [`INPUT_COUNT_RANGE`].
/// * Ensure that all [`UtxoInput`](crate::input::UtxoInput)s are unique.
/// * Ensure that [`Input`]s are sorted lexicographically in their serialized forms.
/// * Contain a number of [`Output]`s within [`OUTPUT_COUNT_RANGE`].
/// * Ensure that [`Output]` amounts to not total above [`IOTA_SUPPLY`].
/// * Ensure that [`Output]`s are sorted lexicographically in their serialized formns.
/// * Ensure that the optional [`Payload`] is of [`IndexationPayload`](crate::payload::indexation::IndexationPayload)
/// type.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct TransactionEssence {
    /// Timestamp of the transaction.
    timestamp: u64,
    /// Node ID to which the access mana of the transaction is pledged.
    access_pledge_id: [u8; PLEDGE_ID_LENGTH],
    /// Node ID to which the consensus mana of the transaction is pledged.
    consensus_pledge_id: [u8; PLEDGE_ID_LENGTH],
    /// Collection of transaction [`Input`]s.
    #[packable(verify_with = validate_inputs)]
    #[packable(unpack_error_with = |e| e.unwrap_packable_or_else(|p| ValidationError::InvalidInputCount(p.into())))]
    inputs: VecPrefix<Input, InputCount>,
    /// Collection of transaction [`Output`]s.
    #[packable(verify_with = validate_outputs)]
    #[packable(unpack_error_with = |e| e.unwrap_packable_or_else(|p| ValidationError::InvalidOutputCount(p.into())))]
    outputs: VecPrefix<Output, OutputCount>,
    /// Optional additional payload.
    #[packable(verify_with = validate_payload)]
    payload: Option<Payload>,
}

impl TransactionEssence {
    /// Create a new [`TransactionEssenceBuilder`] to build a [`TransactionEssence`].
    pub fn builder() -> TransactionEssenceBuilder {
        TransactionEssenceBuilder::new()
    }

    /// Returns the timestamp of a Transaction Essence.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the node ID to which the access mana of the transaction is pledged.
    pub fn access_pledge_id(&self) -> &[u8; PLEDGE_ID_LENGTH] {
        &self.access_pledge_id
    }

    /// Returns the node ID to which the consensus mana of the transaction is pledged.
    pub fn consensus_pledge_id(&self) -> &[u8; PLEDGE_ID_LENGTH] {
        &self.consensus_pledge_id
    }

    /// Returns the inputs of a [`TransactionEssence`].
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Returns the outputs of a [`TransactionEssence`].
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Returns the optional payload of a [`TransactionEssence`].
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

/// A builder to build a [`TransactionEssence`].
#[derive(Debug, Default)]
pub struct TransactionEssenceBuilder {
    timestamp: Option<u64>,
    access_pledge_id: Option<[u8; PLEDGE_ID_LENGTH]>,
    consensus_pledge_id: Option<[u8; PLEDGE_ID_LENGTH]>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    payload: Option<Payload>,
}

impl TransactionEssenceBuilder {
    /// Creates a new [`TransactionEssenceBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a timestamp to a [`TransactionEssenceBuilder`].
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp.replace(timestamp);
        self
    }

    /// Adds an access pledge ID to a [`TransactionEssenceBuilder`].
    #[must_use]
    pub fn with_access_pledge_id(mut self, access_pledge_id: [u8; PLEDGE_ID_LENGTH]) -> Self {
        self.access_pledge_id.replace(access_pledge_id);
        self
    }

    /// Adds a consensus pledge ID to a [`TransactionEssenceBuilder`].
    #[must_use]
    pub fn with_consensus_pledge_id(mut self, consensus_pledge_id: [u8; PLEDGE_ID_LENGTH]) -> Self {
        self.consensus_pledge_id.replace(consensus_pledge_id);
        self
    }

    /// Adds inputs to a [`TransactionEssenceBuilder`]
    #[must_use]
    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Add an input to a [`TransactionEssenceBuilder`].
    #[must_use]
    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Add outputs to a [`TransactionEssenceBuilder`].
    #[must_use]
    pub fn with_outputs(mut self, outputs: Vec<Output>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Add an output to a [`TransactionEssenceBuilder`].
    #[must_use]
    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    /// Add a payload to a [`TransactionEssenceBuilder`].
    #[must_use]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload.replace(payload);
        self
    }

    /// Finishes a [`TransactionEssenceBuilder`] into a [`TransactionEssence`].
    pub fn finish(self) -> Result<TransactionEssence, ValidationError> {
        let timestamp = self
            .timestamp
            .ok_or(ValidationError::MissingBuilderField("timestamp"))?;
        let access_pledge_id = self
            .access_pledge_id
            .ok_or(ValidationError::MissingBuilderField("access_pledge_id"))?;
        let consensus_pledge_id = self
            .consensus_pledge_id
            .ok_or(ValidationError::MissingBuilderField("consensus_pledge_id"))?;

        validate_inputs::<true>(&self.inputs)?;
        validate_outputs::<true>(&self.outputs)?;
        validate_payload::<true>(&self.payload)?;

        Ok(TransactionEssence {
            timestamp,
            access_pledge_id,
            consensus_pledge_id,
            inputs: self.inputs.try_into().map_err(ValidationError::InvalidInputCount)?,
            outputs: self.outputs.try_into().map_err(ValidationError::InvalidOutputCount)?,
            payload: self.payload,
        })
    }
}

fn validate_inputs_unique_utxos(inputs: &[Input]) -> Result<(), ValidationError> {
    for input in inputs.iter() {
        match input {
            Input::Utxo(u) => {
                if inputs.iter().filter(|i| *i == input).count() > 1 {
                    return Err(ValidationError::DuplicateUtxo(u.clone()));
                }
            }
        }
    }

    Ok(())
}

fn validate_inputs_sorted(inputs: &[Input]) -> Result<(), ValidationError> {
    if !is_sorted(inputs.iter().map(PackableExt::pack_to_vec)) {
        Err(ValidationError::TransactionInputsNotSorted)
    } else {
        Ok(())
    }
}

fn validate_output_variant(output: &Output, outputs: &[Output]) -> Result<u64, ValidationError> {
    match output {
        Output::SignatureLockedSingle(single) => validate_single(single, outputs),
        Output::SignatureLockedAsset(_) => Ok(0),
    }
}

fn validate_single(single: &SignatureLockedSingleOutput, outputs: &[Output]) -> Result<u64, ValidationError> {
    if outputs
        .iter()
        .filter(|o| matches!(o, Output::SignatureLockedSingle(s) if s.address() == single.address()))
        .count()
        > 1
    {
        Err(ValidationError::DuplicateAddress(single.address().clone()))
    } else {
        Ok(single.amount())
    }
}

fn validate_output_total(total: u64) -> Result<(), ValidationError> {
    if total > IOTA_SUPPLY {
        Err(ValidationError::InvalidAccumulatedOutput(total as u128))
    } else {
        Ok(())
    }
}

fn validate_outputs_sorted(outputs: &[Output]) -> Result<(), ValidationError> {
    if !is_sorted(outputs.iter().map(|o| o.pack_to_vec())) {
        Err(ValidationError::TransactionOutputsNotSorted)
    } else {
        Ok(())
    }
}

fn validate_inputs<const VERIFY: bool>(inputs: &[Input]) -> Result<(), ValidationError> {
    validate_inputs_unique_utxos(inputs)?;
    validate_inputs_sorted(inputs)?;

    Ok(())
}

fn validate_outputs<const VERIFY: bool>(outputs: &[Output]) -> Result<(), ValidationError> {
    validate_output_total(outputs.iter().try_fold(0u64, |total, output| {
        let amount = validate_output_variant(output, outputs)?;
        total
            .checked_add(amount)
            .ok_or(ValidationError::InvalidAccumulatedOutput(
                total as u128 + amount as u128,
            ))
    })?)?;
    validate_outputs_sorted(outputs)?;

    Ok(())
}

fn validate_payload<const VERIFY: bool>(payload: &Option<Payload>) -> Result<(), ValidationError> {
    match payload {
        None | Some(Payload::Indexation(_)) => Ok(()),
        // Unwrap is fine because we just checked that the Option is not None.
        _ => Err(ValidationError::InvalidPayloadKind(payload.as_ref().unwrap().kind())),
    }
}
