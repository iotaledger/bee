// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::{AliasId, NftId};

///
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, derive_more::From)]
pub enum ChainId {
    ///
    Alias(AliasId),
    ///
    Nft(NftId),
}

impl core::fmt::Display for ChainId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ChainId::Alias(alias) => write!(f, "{}", alias),
            ChainId::Nft(nft) => write!(f, "{}", nft),
        }
    }
}

impl core::fmt::Debug for ChainId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "ChainId({})", self)
    }
}
