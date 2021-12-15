// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::{AliasId, NftId};

use derive_more::From;

///
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, From)]
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
