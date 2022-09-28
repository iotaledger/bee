// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{bee, inx, raw::Raw, return_err_if_none};

/// The [`Block`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The [`BlockId`](bee::BlockId) of the block.
    pub block_id: bee::BlockId,
    /// The complete [`Block`](bee::Block) as raw bytes.
    pub block: Raw<bee::Block>,
}

impl TryFrom<inx::Block> for Block {
    type Error = bee::InxError;

    fn try_from(value: inx::Block) -> Result<Self, Self::Error> {
        Ok(Block {
            block_id: return_err_if_none!(value.block_id).try_into()?,
            block: return_err_if_none!(value.block).data.into(),
        })
    }
}

impl From<Block> for inx::Block {
    fn from(value: Block) -> Self {
        Self {
            block_id: Some(value.block_id.into()),
            block: Some(value.block.into()),
        }
    }
}

/// The [`BlockWithMetadata`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockWithMetadata {
    /// The [`Metadata`](crate::BlockMetadata) of the block.
    pub metadata: crate::BlockMetadata,
    /// The complete [`Block`](bee::Block) as raw bytes.
    pub block: Raw<bee::Block>,
}

impl TryFrom<inx::BlockWithMetadata> for BlockWithMetadata {
    type Error = bee::InxError;

    fn try_from(value: inx::BlockWithMetadata) -> Result<Self, Self::Error> {
        Ok(BlockWithMetadata {
            metadata: return_err_if_none!(value.metadata).try_into()?,
            block: return_err_if_none!(value.block).data.into(),
        })
    }
}

impl From<BlockWithMetadata> for inx::BlockWithMetadata {
    fn from(value: BlockWithMetadata) -> Self {
        Self {
            metadata: Some(value.metadata.into()),
            block: Some(value.block.into()),
        }
    }
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
    pub conflict_reason: bee::ConflictReason,
    /// The whiteflag index of this block inside the milestone.
    pub white_flag_index: u32,
}

impl TryFrom<inx::BlockMetadata> for BlockMetadata {
    type Error = bee::InxError;

    fn try_from(value: inx::BlockMetadata) -> Result<Self, Self::Error> {
        let ledger_inclusion_state = value.ledger_inclusion_state().into();
        let conflict_reason = value.conflict_reason().into();

        let parents = value
            .parents
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BlockMetadata {
            block_id: return_err_if_none!(value.block_id).try_into()?,
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

impl From<inx::LedgerInclusionState> for LedgerInclusionState {
    fn from(value: inx::LedgerInclusionState) -> Self {
        use crate::inx::LedgerInclusionState::*;
        match value {
            NoTransaction => LedgerInclusionState::NoTransaction,
            Included => LedgerInclusionState::Included,
            Conflicting => LedgerInclusionState::Conflicting,
        }
    }
}

impl From<LedgerInclusionState> for inx::LedgerInclusionState {
    fn from(value: LedgerInclusionState) -> Self {
        match value {
            LedgerInclusionState::NoTransaction => Self::NoTransaction,
            LedgerInclusionState::Included => Self::Included,
            LedgerInclusionState::Conflicting => Self::Conflicting,
        }
    }
}

impl From<BlockMetadata> for inx::BlockMetadata {
    fn from(value: BlockMetadata) -> Self {
        Self {
            block_id: Some(value.block_id.into()),
            parents: value.parents.into_vec().into_iter().map(Into::into).collect(),
            solid: value.is_solid,
            should_promote: value.should_promote,
            should_reattach: value.should_reattach,
            referenced_by_milestone_index: value.referenced_by_milestone_index,
            milestone_index: value.milestone_index,
            ledger_inclusion_state: inx::block_metadata::LedgerInclusionState::from(value.ledger_inclusion_state)
                .into(),
            conflict_reason: inx::block_metadata::ConflictReason::from(value.conflict_reason).into(),
            white_flag_index: value.white_flag_index,
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedgerInclusionState {
    NoTransaction,
    Included,
    Conflicting,
}
