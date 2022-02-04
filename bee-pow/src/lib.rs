// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains traits and implementations to provide and score Proof of Work.
//! RFC <https://github.com/Wollac/protocol-rfcs/blob/message-pow/text/0024-message-pow/0024-message-pow.md>.

#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]

pub mod providers;
pub mod score;
