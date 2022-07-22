// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq)]
pub enum LedgerInclusionState {
    NoTransaction,
    Included,
    Conflicting,
}

/// The metadata for a block with a given [`BlockId`](bee::BlockId).
#[derive(Clone, Debug, PartialEq)]
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

        let mut parents = Vec::with_capacity(value.parents.len());
        for parent in value.parents {
            parents.push(parent.try_into()?);
        }

        Ok(BlockMetadata {
            block_id: value
                .block_id
                .ok_or(Self::Error::MissingField("block_id"))?
                .try_into()?,
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
        match value {
            proto::block_metadata::LedgerInclusionState::NoTransaction => LedgerInclusionState::NoTransaction,
            proto::block_metadata::LedgerInclusionState::Included => LedgerInclusionState::Included,
            proto::block_metadata::LedgerInclusionState::Conflicting => LedgerInclusionState::Conflicting,
        }
    }
}
