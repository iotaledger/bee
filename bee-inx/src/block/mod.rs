// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod responses;

pub use self::responses::*;
use crate::client::Inx;

//   rpc ListenToBlocks(NoParams) returns (stream Block);
//   rpc ListenToSolidBlocks(NoParams) returns (stream BlockMetadata);
//   rpc ListenToReferencedBlocks(NoParams) returns (stream BlockMetadata);
//   rpc SubmitBlock(RawBlock) returns (BlockId);
//   rpc ReadBlock(BlockId) returns (RawBlock);
//   rpc ReadBlockMetadata(BlockId) returns (BlockMetadata);

impl Inx {
    // TODO
}
