// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::maybe_missing;

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedgerInclusionState {
    NoTransaction,
    Included,
    Conflicting,
}

/// The metadata for a block with a given [`BlockId`](bee::BlockId).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockMetadata {
    /// The id of the block.
    pub block_id: bee::BlockId,
    /// The parents of the block.
    pub parents: Box<[bee::BlockId]>,
    /// Status of the solidification process.
    pub is_solid: bool,
    /// Indicates that the block should be promoted.
    pub should_promote: bool,
    /// Indicates that the block should be reattached.
    pub should_reattach: bool,
    /// The milestone that referenced the block.
    pub referenced_by_milestone_index: u32,
    /// The corresponding milestone index.
    pub milestone_index: u32,
    /// Indicates if a block is part of the ledger state or not.
    pub ledger_inclusion_state: LedgerInclusionState,
    /// Indicates if a conflict occurred, and if so holds information about the reason for the conflict.
    pub conflict_reason: bee::semantic::ConflictReason,
    /// The whiteflag index of this block inside the milestone.
    pub white_flag_index: u32,
}

impl TryFrom<proto::BlockMetadata> for BlockMetadata {
    type Error = bee::InxError;

    fn try_from(value: proto::BlockMetadata) -> Result<Self, Self::Error> {
        let ledger_inclusion_state = value.ledger_inclusion_state().into();
        let conflict_reason = value.conflict_reason().into();

        let parents = value
            .parents
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BlockMetadata {
            block_id: maybe_missing!(value.block_id).try_into()?,
            parents: parents.into_boxed_slice(),
            is_solid: value.solid,
            should_promote: value.should_promote,
            should_reattach: value.should_reattach,
            referenced_by_milestone_index: value.referenced_by_milestone_index,
            milestone_index: value.milestone_index,
            ledger_inclusion_state,
            conflict_reason,
            white_flag_index: value.white_flag_index,
        })
    }
}

impl From<proto::block_metadata::LedgerInclusionState> for LedgerInclusionState {
    fn from(value: proto::block_metadata::LedgerInclusionState) -> Self {
        use proto::block_metadata::LedgerInclusionState::*;
        match value {
            NoTransaction => LedgerInclusionState::NoTransaction,
            Included => LedgerInclusionState::Included,
            Conflicting => LedgerInclusionState::Conflicting,
        }
    }
}

impl From<LedgerInclusionState> for proto::block_metadata::LedgerInclusionState {
    fn from(value: LedgerInclusionState) -> Self {
        match value {
            LedgerInclusionState::NoTransaction => Self::NoTransaction,
            LedgerInclusionState::Included => Self::Included,
            LedgerInclusionState::Conflicting => Self::Conflicting,
        }
    }
}

impl From<BlockMetadata> for proto::BlockMetadata {
    fn from(value: BlockMetadata) -> Self {
        Self {
            block_id: Some(value.block_id.into()),
            parents: value.parents.into_vec().into_iter().map(Into::into).collect(),
            solid: value.is_solid,
            should_promote: value.should_promote,
            should_reattach: value.should_reattach,
            referenced_by_milestone_index: value.referenced_by_milestone_index,
            milestone_index: value.milestone_index,
            ledger_inclusion_state: proto::block_metadata::LedgerInclusionState::from(value.ledger_inclusion_state)
                .into(),
            conflict_reason: proto::block_metadata::ConflictReason::from(value.conflict_reason).into(),
            white_flag_index: value.white_flag_index,
        }
    }
}
